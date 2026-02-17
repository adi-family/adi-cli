use cli::user_config::UserConfig;
use dialoguer::console::{style, Key, Term};
use lib_console_output::blocks::{KeyValue, Renderable, Section};
use lib_console_output::theme;
use lib_console_output::theme::generated::THEMES;
use lib_console_output::{out_info, out_success};

struct ThemeEntry {
    id: &'static str,
    name: &'static str,
    accent_hex: &'static str,
    accent: u8,
    text: u8,
}

fn build_entries() -> Vec<ThemeEntry> {
    THEMES
        .iter()
        .map(|t| ThemeEntry {
            id: t.id,
            name: t.name,
            accent_hex: t.dark.accent,
            accent: theme::hex_to_ansi256(t.dark.accent),
            text: theme::hex_to_ansi256(t.dark.text),
        })
        .collect()
}

fn render_list(term: &Term, entries: &[ThemeEntry], cursor: usize, clear: bool) {
    if clear {
        let _ = term.clear_last_lines(entries.len());
    }

    let max_name = entries.iter().map(|e| e.name.len()).max().unwrap_or(0);

    for (i, entry) in entries.iter().enumerate() {
        let selected = i == cursor;

        let label = format!(
            "{}{} {}",
            entry.name,
            " ".repeat(max_name.saturating_sub(entry.name.len())),
            entry.accent_hex,
        );

        let line = if selected {
            // Full row with accent foreground
            let padded = format!(" > \u{2588}\u{2588} {} ", label);
            format!("{}", style(padded).color256(entry.accent).bold(),)
        } else {
            // Swatch in accent, name in theme text
            format!(
                "   {}{}",
                style("\u{2588}\u{2588}").color256(entry.accent),
                style(format!(" {}", label)).color256(entry.text),
            )
        };
        lib_console_output::fg_println!("  {line}");
    }
}

pub(crate) fn cmd_theme() -> anyhow::Result<()> {
    let active = theme::active();

    Section::new("Theme").width(50).print();

    KeyValue::new()
        .entry("Current", theme::brand_bold(&active.name).to_string())
        .entry("ID", theme::muted(&active.id).to_string())
        .entry("Accent", theme::brand(&active.dark.accent).to_string())
        .print();

    println!();
    out_info!("Select a new theme:");

    let entries = build_entries();
    let mut cursor = entries.iter().position(|e| e.id == active.id).unwrap_or(0);

    let term = Term::stdout();
    render_list(&term, &entries, cursor, false);

    loop {
        match term.read_key() {
            Ok(Key::ArrowUp | Key::Char('k')) => {
                cursor = if cursor == 0 {
                    entries.len() - 1
                } else {
                    cursor - 1
                };
                render_list(&term, &entries, cursor, true);
            }
            Ok(Key::ArrowDown | Key::Char('j')) => {
                cursor = (cursor + 1) % entries.len();
                render_list(&term, &entries, cursor, true);
            }
            Ok(Key::Enter) => {
                let _ = term.clear_last_lines(entries.len());
                let entry = &entries[cursor];

                if entry.id == active.id {
                    lib_console_output::fg_println!(
                        "{} {}",
                        theme::success(theme::icons::SUCCESS),
                        theme::foreground(entry.name),
                    );
                    out_info!("Theme unchanged.");
                    return Ok(());
                }

                let mut config = UserConfig::load()?;
                config.theme = Some(entry.id.to_string());
                config.save()?;

                lib_console_output::fg_println!(
                    "{} {}",
                    style(theme::icons::SUCCESS).color256(entry.accent),
                    style(entry.name).color256(entry.accent).bold(),
                );
                out_success!("Theme saved — restart CLI to apply.");
                return Ok(());
            }
            Ok(Key::Escape | Key::Char('q')) => {
                let _ = term.clear_last_lines(entries.len());
                lib_console_output::fg_println!(
                    "{} {}",
                    theme::error(theme::icons::ERROR),
                    theme::foreground("Cancelled")
                );
                return Ok(());
            }
            _ => {}
        }
    }
}
