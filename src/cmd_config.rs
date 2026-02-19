use cli::user_config::UserConfig;
use dialoguer::console::{style, Key, Term};
use lib_console_output::blocks::{KeyValue, Renderable, Section};
use lib_console_output::theme;
use lib_console_output::{out_info, out_success};

use crate::args::ConfigCommands;

pub(crate) async fn cmd_config(command: Option<ConfigCommands>) -> anyhow::Result<()> {
    match command {
        Some(ConfigCommands::Show) => cmd_config_show(),
        Some(ConfigCommands::PowerUser { enable }) => {
            let value = match enable.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => true,
                "false" | "0" | "no" | "off" => false,
                _ => anyhow::bail!("Invalid value '{}'. Use 'true' or 'false'.", enable),
            };
            cmd_config_power_user_set(value)
        }
        None => {
            // No subcommand: interactive in TTY, show otherwise
            if UserConfig::is_interactive() {
                cmd_config_interactive()
            } else {
                cmd_config_show()
            }
        }
    }
}

fn cmd_config_show() -> anyhow::Result<()> {
    let config = UserConfig::load()?;
    let config_path = UserConfig::config_path()?;

    Section::new("Configuration").width(50).print();

    let power_user_status = match config.power_user {
        Some(true) => theme::success("enabled").to_string(),
        Some(false) => theme::muted("disabled").to_string(),
        None => theme::muted("default (disabled)").to_string(),
    };

    let language_status = config
        .language
        .as_deref()
        .map(|l| theme::foreground(l).to_string())
        .unwrap_or_else(|| theme::muted("not set").to_string());

    let theme_status = config
        .theme
        .as_deref()
        .map(|t| theme::brand(t).to_string())
        .unwrap_or_else(|| theme::muted("default").to_string());

    KeyValue::new()
        .entry("Power User", power_user_status)
        .entry("Language", language_status)
        .entry("Theme", theme_status)
        .entry(
            "Config File",
            theme::muted(config_path.display()).to_string(),
        )
        .print();

    Ok(())
}

struct ConfigOption {
    key: &'static str,
    label: &'static str,
    value_fn: fn(&UserConfig) -> String,
}

const CONFIG_OPTIONS: &[ConfigOption] = &[
    ConfigOption {
        key: "power_user",
        label: "Power User",
        value_fn: |c| match c.power_user {
            Some(true) => "enabled".to_string(),
            Some(false) | None => "disabled".to_string(),
        },
    },
];

fn cmd_config_interactive() -> anyhow::Result<()> {
    let config = UserConfig::load()?;

    Section::new("Configuration").width(50).print();
    out_info!("Use arrows to navigate, Enter to toggle, q to quit");
    println!();

    let term = Term::stdout();
    let mut cursor = 0;

    render_config_list(&term, &config, cursor, false);

    loop {
        match term.read_key() {
            Ok(Key::ArrowUp | Key::Char('k')) => {
                cursor = if cursor == 0 {
                    CONFIG_OPTIONS.len() - 1
                } else {
                    cursor - 1
                };
                render_config_list(&term, &config, cursor, true);
            }
            Ok(Key::ArrowDown | Key::Char('j')) => {
                cursor = (cursor + 1) % CONFIG_OPTIONS.len();
                render_config_list(&term, &config, cursor, true);
            }
            Ok(Key::Enter | Key::Char(' ')) => {
                let _ = term.clear_last_lines(CONFIG_OPTIONS.len());
                let opt = &CONFIG_OPTIONS[cursor];
                
                match opt.key {
                    "power_user" => {
                        toggle_power_user()?;
                    }
                    _ => {}
                }
                
                // Reload config and re-render
                let config = UserConfig::load()?;
                render_config_list(&term, &config, cursor, false);
            }
            Ok(Key::Escape | Key::Char('q')) => {
                let _ = term.clear_last_lines(CONFIG_OPTIONS.len());
                lib_console_output::fg_println!(
                    "{} {}",
                    theme::muted(theme::icons::INFO),
                    theme::foreground("Done")
                );
                return Ok(());
            }
            _ => {}
        }
    }
}

fn render_config_list(term: &Term, config: &UserConfig, cursor: usize, clear: bool) {
    if clear {
        let _ = term.clear_last_lines(CONFIG_OPTIONS.len());
    }

    let max_label = CONFIG_OPTIONS.iter().map(|o| o.label.len()).max().unwrap_or(0);

    for (i, opt) in CONFIG_OPTIONS.iter().enumerate() {
        let selected = i == cursor;
        let value = (opt.value_fn)(config);

        let value_styled = if value == "enabled" {
            theme::success(&value).to_string()
        } else {
            theme::muted(&value).to_string()
        };

        let label_padded = format!(
            "{}{}",
            opt.label,
            " ".repeat(max_label.saturating_sub(opt.label.len()))
        );

        let line = if selected {
            format!(
                " {} {} : {}",
                style(">").bold(),
                style(&label_padded).bold(),
                value_styled
            )
        } else {
            format!(
                "   {} : {}",
                theme::foreground(&label_padded),
                value_styled
            )
        };

        lib_console_output::fg_println!("{line}");
    }
}

fn cmd_config_power_user_set(enable: bool) -> anyhow::Result<()> {
    let mut config = UserConfig::load()?;
    config.power_user = Some(enable);
    config.save()?;

    if enable {
        out_success!("Power user mode enabled.");
        out_info!("Advanced features and verbose output are now active.");
    } else {
        out_success!("Power user mode disabled.");
    }

    Ok(())
}

fn toggle_power_user() -> anyhow::Result<()> {
    let mut config = UserConfig::load()?;
    let current = config.power_user.unwrap_or(false);
    let new_value = !current;
    config.power_user = Some(new_value);
    config.save()?;

    if new_value {
        out_success!("Power user mode enabled.");
        out_info!("Advanced features and verbose output are now active.");
    } else {
        out_success!("Power user mode disabled.");
    }

    Ok(())
}
