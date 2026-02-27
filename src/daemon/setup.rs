use anyhow::{bail, Context, Result};
use lib_console_output::theme;
use std::process::Command;

const SUDOERS_PATH: &str = "/etc/sudoers.d/adi-daemon";

/// Run the daemon setup: create adi-root user, configure sudoers, prepare directories.
pub async fn run_setup() -> Result<()> {
    verify_platform()?;
    verify_interactive()?;

    println!(
        "{} ADI daemon setup — creates system user and privilege escalation rules",
        theme::icons::INFO,
    );
    println!(
        "  {}",
        theme::muted("You will be prompted for your sudo password once."),
    );
    println!();

    // Trigger sudo credential caching up front so subsequent commands don't re-prompt
    warm_sudo()?;

    let root_user = crate::clienv::daemon_root_user();

    setup_user(&root_user)?;
    setup_sudoers(&root_user)?;

    #[cfg(target_os = "macos")]
    setup_resolver_dir()?;

    println!();
    println!(
        "{} Daemon setup complete",
        theme::icons::SUCCESS,
    );
    println!(
        "  Run {} to start the daemon",
        theme::bold("adi daemon start"),
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Prerequisite checks
// ---------------------------------------------------------------------------

fn verify_platform() -> Result<()> {
    if cfg!(not(any(target_os = "macos", target_os = "linux"))) {
        bail!("Daemon setup is only supported on macOS and Linux");
    }
    Ok(())
}

fn verify_interactive() -> Result<()> {
    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        bail!("Setup requires an interactive terminal (stdin must be a TTY)");
    }
    Ok(())
}

/// Prompt for sudo password so subsequent sudo calls reuse the cached credential.
fn warm_sudo() -> Result<()> {
    let status = Command::new("sudo")
        .args(["-v"])
        .status()
        .context("Failed to run sudo")?;

    if !status.success() {
        bail!("sudo authentication failed");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// User creation
// ---------------------------------------------------------------------------

fn user_exists(name: &str) -> bool {
    Command::new("id")
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn setup_user(name: &str) -> Result<()> {
    if user_exists(name) {
        println!(
            "  {} User {} already exists",
            theme::icons::SUCCESS,
            theme::bold(name),
        );
        return Ok(());
    }

    println!(
        "  {} Creating system user {}...",
        theme::icons::IN_PROGRESS,
        theme::bold(name),
    );

    #[cfg(target_os = "macos")]
    create_user_macos(name)?;

    #[cfg(target_os = "linux")]
    create_user_linux(name)?;

    println!(
        "  {} Created user {}",
        theme::icons::SUCCESS,
        theme::bold(name),
    );
    Ok(())
}

#[cfg(target_os = "macos")]
fn create_user_macos(name: &str) -> Result<()> {
    let uid = find_available_uid_macos()?;
    let uid_str = uid.to_string();

    let dscl_steps: &[&[&str]] = &[
        &["dscl", ".", "-create", &format!("/Users/{name}")],
        &["dscl", ".", "-create", &format!("/Users/{name}"), "UserShell", "/usr/bin/false"],
        &["dscl", ".", "-create", &format!("/Users/{name}"), "RealName", "ADI Privileged Daemon"],
        &["dscl", ".", "-create", &format!("/Users/{name}"), "UniqueID", &uid_str],
        &["dscl", ".", "-create", &format!("/Users/{name}"), "PrimaryGroupID", "20"],
        &["dscl", ".", "-create", &format!("/Users/{name}"), "NFSHomeDirectory", "/var/empty"],
        &["dscl", ".", "-create", &format!("/Users/{name}"), "IsHidden", "1"],
    ];

    for args in dscl_steps {
        run_sudo(args).with_context(|| format!("dscl step failed: {:?}", args))?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn find_available_uid_macos() -> Result<u32> {
    let output = Command::new("dscl")
        .args([".", "-list", "/Users", "UniqueID"])
        .output()
        .context("Failed to list existing UIDs")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let taken: std::collections::HashSet<u32> = stdout
        .lines()
        .filter_map(|line| line.split_whitespace().last()?.parse().ok())
        .collect();

    (300..400)
        .find(|uid| !taken.contains(uid))
        .context("No available UID in system range 300-399")
}

#[cfg(target_os = "linux")]
fn create_user_linux(name: &str) -> Result<()> {
    run_sudo(&[
        "useradd", "-r", "-s", "/usr/sbin/nologin", "-d", "/nonexistent", "-M", name,
    ])
    .context("Failed to create Linux system user")
}

// ---------------------------------------------------------------------------
// Sudoers
// ---------------------------------------------------------------------------

fn setup_sudoers(root_user: &str) -> Result<()> {
    if std::path::Path::new(SUDOERS_PATH).exists() {
        println!(
            "  {} Sudoers file already exists ({})",
            theme::icons::SUCCESS,
            theme::muted(SUDOERS_PATH),
        );
        return Ok(());
    }

    println!(
        "  {} Configuring sudoers rules...",
        theme::icons::IN_PROGRESS,
    );

    let admin_group = if cfg!(target_os = "macos") {
        "%admin"
    } else {
        "%sudo"
    };

    let content = format!(
        "# ADI Daemon privilege escalation\n\
         # Allow admin users to switch to {root_user} without password\n\
         {admin_group} ALL=({root_user}) NOPASSWD: ALL\n\
         \n\
         # Allow {root_user} to run any command as root without password\n\
         {root_user} ALL=(ALL) NOPASSWD: ALL\n"
    );

    write_sudoers_safe(&content)?;

    println!(
        "  {} Sudoers rules installed ({})",
        theme::icons::SUCCESS,
        theme::muted(SUDOERS_PATH),
    );
    Ok(())
}

/// Write sudoers content through a validated temp file.
fn write_sudoers_safe(content: &str) -> Result<()> {
    let tmp = "/tmp/adi-daemon-sudoers.tmp";

    // Write to temp
    std::fs::write(tmp, content).context("Failed to write temp sudoers file")?;

    // Validate syntax
    let check = Command::new("sudo")
        .args(["visudo", "-cf", tmp])
        .output()
        .context("Failed to run visudo validation")?;

    if !check.status.success() {
        let stderr = String::from_utf8_lossy(&check.stderr);
        std::fs::remove_file(tmp).ok();
        bail!("Sudoers syntax validation failed: {}", stderr.trim());
    }

    // Set correct ownership, permissions, and move into place
    run_sudo(&["chown", "root:wheel", tmp])?;
    run_sudo(&["chmod", "0440", tmp])?;
    run_sudo(&["mv", tmp, SUDOERS_PATH])?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Resolver directory (macOS)
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
fn setup_resolver_dir() -> Result<()> {
    let path = std::path::Path::new("/etc/resolver");
    if path.exists() {
        println!(
            "  {} /etc/resolver directory exists",
            theme::icons::SUCCESS,
        );
        return Ok(());
    }

    println!(
        "  {} Creating /etc/resolver directory...",
        theme::icons::IN_PROGRESS,
    );
    run_sudo(&["mkdir", "-p", "/etc/resolver"])?;
    println!(
        "  {} Created /etc/resolver",
        theme::icons::SUCCESS,
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn run_sudo(args: &[&str]) -> Result<()> {
    let status = Command::new("sudo")
        .args(args)
        .status()
        .with_context(|| format!("Failed to execute: sudo {}", args.join(" ")))?;

    if !status.success() {
        bail!("Command failed (exit {}): sudo {}", status, args.join(" "));
    }
    Ok(())
}
