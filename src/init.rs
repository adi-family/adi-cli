use cli::plugin_registry::PluginManager;
use cli::user_config::UserConfig;
use dialoguer::{theme::ColorfulTheme, Select};
use lib_console_output::{theme, out_info, out_success, out_warn};
use lib_i18n_core::{init_global, I18n};

/// Initialize the ADI theme from env var or user config.
///
/// Priority: ADI_THEME env var > config file theme > default ("indigo").
pub(crate) fn initialize_theme() {
    let theme_id = cli::clienv::theme()
        .or_else(|| UserConfig::load().ok().and_then(|c| c.theme))
        .unwrap_or_else(|| lib_console_output::theme::generated::DEFAULT_THEME.to_string());
    tracing::trace!(theme = %theme_id, "Initializing theme");
    lib_console_output::theme::init(&theme_id);
}

pub(crate) async fn initialize_i18n(lang_override: Option<&str>) -> anyhow::Result<()> {
    tracing::trace!(lang_override = ?lang_override, "Initializing i18n");
    let mut config = UserConfig::load()?;

    // Detect language with priority:
    // 1. CLI --lang flag (highest priority)
    // 2. ADI_LANG environment variable
    // 3. Saved user preference
    // 4. System LANG environment variable
    // 5. Interactive prompt on first run (if TTY)
    // 6. Default to en-US
    let user_lang = if let Some(lang) = lang_override {
        tracing::trace!(lang = %lang, "Language from CLI --lang flag");
        lang.to_string()
    } else if let Some(env_lang) = cli::clienv::lang() {
        tracing::trace!(lang = %env_lang, "Language from ADI_LANG env var");
        env_lang
    } else if let Some(saved_lang) = &config.language {
        tracing::trace!(lang = %saved_lang, "Language from saved user config");
        saved_lang.clone()
    } else if let Some(system_lang) = cli::clienv::system_lang() {
        let lang = system_lang
            .split('.')
            .next()
            .map(|s| s.replace('_', "-"))
            .unwrap_or_else(|| "en-US".to_string());
        tracing::trace!(system_lang = %system_lang, resolved = %lang, "Language from system LANG env var");
        lang
    } else if UserConfig::is_first_run()? && UserConfig::is_interactive() {
        tracing::trace!("First run, prompting for language selection");
        let selected_lang = prompt_language_selection().await?;

        config.language = Some(selected_lang.clone());
        config.save()?;

        out_success!("Language set to: {}", selected_lang);
        out_info!("{}", theme::muted("You can change this later by setting ADI_LANG environment variable or using --lang flag"));

        selected_lang
    } else {
        tracing::trace!("Defaulting to en-US");
        "en-US".to_string()
    };

    tracing::trace!(lang = %user_lang, "Selected language");

    // Initialize i18n with direct FTL file loading (no plugin service registry needed)
    let mut i18n = I18n::new_standalone();

    // Load embedded English translations as fallback (always available)
    let _ = i18n.load_embedded("en-US", include_str!("../plugins/en-US/messages.ftl"));
    tracing::trace!("Loaded embedded en-US translations");

    // Try to load additional language from installed plugins
    if user_lang != "en-US" {
        let translation_id = format!("adi.cli.{}", user_lang);
        tracing::trace!(translation_id = %translation_id, "Looking for translation plugin");

        let plugins_dir = lib_plugin_host::PluginConfig::default_plugins_dir();

        let plugin_dir = plugins_dir.join(&translation_id);
        let ftl_loaded = if let Some(ftl_path) = find_messages_ftl(&plugin_dir) {
            tracing::trace!(path = %ftl_path.display(), "Found FTL file");
            if let Ok(ftl_content) = std::fs::read_to_string(&ftl_path) {
                let ok = i18n.load_embedded(&user_lang, &ftl_content).is_ok();
                tracing::trace!(loaded = ok, "Loaded translation FTL");
                ok
            } else {
                tracing::trace!("Failed to read FTL file");
                false
            }
        } else {
            tracing::trace!("No FTL file found for translation plugin");
            false
        };

        if !ftl_loaded && should_check_translation(&plugins_dir, &translation_id) {
            tracing::trace!(translation_id = %translation_id, "Attempting to install translation plugin");
            out_info!("{}", theme::muted(format!("Installing {} translation plugin...", user_lang)));

            mark_translation_checked(&plugins_dir, &translation_id);

            let manager = PluginManager::new();
            if manager.install_plugin(&translation_id, None).await.is_ok() {
                tracing::trace!("Translation plugin installed, loading FTL");
                if let Some(ftl_path) = find_messages_ftl(&plugin_dir) {
                    if let Ok(ftl_content) = std::fs::read_to_string(&ftl_path) {
                        let _ = i18n.load_embedded(&user_lang, &ftl_content);
                    }
                }
            } else {
                tracing::trace!("Translation plugin not available");
                out_warn!("Translation plugin {} not available, using English", translation_id);
            }
        }
    }

    // Try to set requested language, fallback to en-US if not available
    if i18n.set_language(&user_lang).is_err() {
        tracing::trace!(lang = %user_lang, "Language not available, falling back to en-US");
        let _ = i18n.set_language("en-US");
    }
    init_global(i18n);
    tracing::trace!("i18n initialized globally");

    Ok(())
}

/// Discover available translation languages from the plugin registry.
///
/// Falls back to scanning installed plugins, then to just en-US (built-in).
async fn get_available_languages() -> Vec<(String, String)> {
    tracing::trace!("Discovering available languages");
    let mut languages = vec![("en-US".to_string(), "English".to_string())];

    let manager = PluginManager::new();
    if let Ok(plugins) = manager.list_plugins().await {
        for plugin in plugins {
            if plugin.plugin_type == "translation" {
                if let Some(lang_code) = plugin.id.strip_prefix("adi.cli.") {
                    if lang_code != "en-US" {
                        let display_name = plugin
                            .name
                            .strip_prefix("ADI CLI - ")
                            .unwrap_or(&plugin.name)
                            .to_string();
                        tracing::trace!(lang = %lang_code, name = %display_name, "Found translation plugin in registry");
                        languages.push((lang_code.to_string(), display_name));
                    }
                }
            }
        }
        return languages;
    }

    tracing::trace!("Registry unreachable, scanning installed plugins for translations");

    // Registry unreachable — scan installed plugins for translation metadata
    let plugins_dir = lib_plugin_host::PluginConfig::default_plugins_dir();

    if let Ok(mut entries) = tokio::fs::read_dir(&plugins_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(lang_code) = name.strip_prefix("adi.cli.") {
                if lang_code == "en-US" {
                    continue;
                }
                let version_file = entry.path().join(".version");
                let display_name = tokio::fs::read_to_string(&version_file)
                    .await
                    .ok()
                    .and_then(|version| {
                        let manifest = entry.path().join(version.trim()).join("plugin.toml");
                        std::fs::read_to_string(&manifest).ok()
                    })
                    .and_then(|content| {
                        content.parse::<toml::Table>().ok().and_then(|table| {
                            table
                                .get("translation")
                                .and_then(|t| t.get("language_name"))
                                .and_then(|n| n.as_str())
                                .map(String::from)
                        })
                    })
                    .unwrap_or_else(|| lang_code.to_string());
                tracing::trace!(lang = %lang_code, name = %display_name, "Found installed translation plugin");
                languages.push((lang_code.to_string(), display_name));
            }
        }
    }

    tracing::trace!(count = languages.len(), "Available languages discovered");
    languages
}

/// Prompt user to select their preferred language interactively.
///
/// Fetches the language list from plugins. If only en-US is available, skips the prompt.
async fn prompt_language_selection() -> anyhow::Result<String> {
    let languages = get_available_languages().await;

    if languages.len() <= 1 {
        tracing::trace!("Only en-US available, skipping language prompt");
        return Ok("en-US".to_string());
    }

    out_info!("{}", theme::brand_bold("Welcome to ADI! 🎉"));
    out_info!("Please select your preferred language:");

    let items: Vec<String> = languages
        .iter()
        .map(|(code, name)| format!("{} ({})", name, code))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()?;

    tracing::trace!(selected = %languages[selection].0, "User selected language");
    Ok(languages[selection].0.clone())
}

/// Check if we should attempt to download a translation plugin (once per day).
fn should_check_translation(plugins_dir: &std::path::Path, translation_id: &str) -> bool {
    let stamp = plugins_dir.join(format!(".{}.last-check", translation_id));
    let should = match std::fs::metadata(&stamp) {
        Ok(meta) => meta
            .modified()
            .ok()
            .and_then(|t| t.elapsed().ok())
            .map_or(true, |age| age > std::time::Duration::from_secs(86400)),
        Err(_) => true,
    };
    tracing::trace!(translation_id = %translation_id, should_check = should, "Translation check status");
    should
}

/// Record that we just attempted to install a translation plugin.
fn mark_translation_checked(plugins_dir: &std::path::Path, translation_id: &str) {
    let stamp = plugins_dir.join(format!(".{}.last-check", translation_id));
    let _ = std::fs::create_dir_all(plugins_dir);
    let _ = std::fs::write(&stamp, []);
    tracing::trace!(translation_id = %translation_id, "Marked translation check timestamp");
}

/// Find the messages.ftl file in a plugin directory (handles versioned directories)
fn find_messages_ftl(plugin_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    tracing::trace!(dir = %plugin_dir.display(), "Searching for messages.ftl");

    let version_file = plugin_dir.join(".version");
    if version_file.exists() {
        if let Ok(version) = std::fs::read_to_string(&version_file) {
            let version = version.trim();
            let ftl_path = plugin_dir.join(version).join("messages.ftl");
            if ftl_path.exists() {
                tracing::trace!(path = %ftl_path.display(), "Found versioned messages.ftl");
                return Some(ftl_path);
            }
        }
    }

    let direct_ftl = plugin_dir.join("messages.ftl");
    if direct_ftl.exists() {
        tracing::trace!(path = %direct_ftl.display(), "Found direct messages.ftl");
        return Some(direct_ftl);
    }

    if let Ok(entries) = std::fs::read_dir(plugin_dir) {
        for entry in entries.flatten() {
            let subdir = entry.path();
            if subdir.is_dir() {
                let ftl_path = subdir.join("messages.ftl");
                if ftl_path.exists() {
                    tracing::trace!(path = %ftl_path.display(), "Found messages.ftl in subdirectory");
                    return Some(ftl_path);
                }
            }
        }
    }

    tracing::trace!("No messages.ftl found");
    None
}
