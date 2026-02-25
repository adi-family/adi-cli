use cli::plugin_registry::PluginManager;
use cli::user_config::UserConfig;
use lib_console_output::{theme, out_info, out_success, out_warn};
use lib_console_output::input::Select;
use lib_i18n_core::{init_global, I18n};

pub(crate) fn initialize_theme() {
    let theme_id = cli::clienv::theme()
        .or_else(|| UserConfig::load().ok().and_then(|c| c.theme))
        .unwrap_or_else(|| lib_console_output::theme::generated::DEFAULT_THEME.to_string());
    tracing::trace!(theme = %theme_id, "Initializing theme");
    lib_console_output::theme::init(&theme_id);
}

async fn resolve_language(
    lang_override: Option<&str>,
    config: &mut UserConfig,
) -> anyhow::Result<String> {
    if let Some(lang) = lang_override {
        tracing::trace!(lang = %lang, "Language from CLI --lang flag");
        return Ok(lang.to_string());
    }
    if let Some(env_lang) = cli::clienv::lang() {
        tracing::trace!(lang = %env_lang, "Language from ADI_LANG env var");
        return Ok(env_lang);
    }
    if let Some(saved_lang) = &config.language {
        tracing::trace!(lang = %saved_lang, "Language from saved user config");
        return Ok(saved_lang.clone());
    }
    if let Some(system_lang) = cli::clienv::system_lang() {
        let lang = system_lang
            .split('.')
            .next()
            .map(|s| s.replace('_', "-"))
            .unwrap_or_else(|| "en-US".to_string());
        tracing::trace!(system_lang = %system_lang, resolved = %lang, "Language from system LANG env var");
        return Ok(lang);
    }
    if UserConfig::is_first_run()? && UserConfig::is_interactive() {
        tracing::trace!("First run, prompting for language selection");
        let selected_lang = prompt_language_selection().await?;

        config.language = Some(selected_lang.clone());
        config.save()?;

        out_success!("Language set to: {}", selected_lang);
        out_info!("{}", theme::muted("You can change this later by setting ADI_LANG environment variable or using --lang flag"));

        return Ok(selected_lang);
    }
    tracing::trace!("Defaulting to en-US");
    Ok("en-US".to_string())
}

pub(crate) async fn initialize_i18n(lang_override: Option<&str>) -> anyhow::Result<()> {
    tracing::trace!(lang_override = ?lang_override, "Initializing i18n");
    let mut config = UserConfig::load()?;

    let user_lang = resolve_language(lang_override, &mut config).await?;
    tracing::trace!(lang = %user_lang, "Selected language");

    let mut i18n = I18n::new_standalone();
    let _ = i18n.load_embedded("en-US", include_str!("../plugins/en-US/messages.ftl"));
    tracing::trace!("Loaded embedded en-US translations");

    if user_lang != "en-US" {
        load_translation(&mut i18n, &user_lang).await;
    }

    if i18n.set_language(&user_lang).is_err() {
        tracing::trace!(lang = %user_lang, "Language not available, falling back to en-US");
        let _ = i18n.set_language("en-US");
    }
    init_global(i18n);
    tracing::trace!("i18n initialized globally");

    Ok(())
}

async fn load_translation(i18n: &mut I18n, lang: &str) {
    let translation_id = format!("{}{}", cli::clienv::CLI_PLUGIN_PREFIX, lang);
    tracing::trace!(translation_id = %translation_id, "Looking for translation plugin");

    let plugins_dir = lib_plugin_host::PluginConfig::default_plugins_dir();
    let plugin_dir = plugins_dir.join(&translation_id);

    if try_load_ftl(i18n, lang, &plugin_dir) {
        return;
    }

    if !should_check_translation(&plugins_dir, &translation_id) {
        return;
    }

    tracing::trace!(translation_id = %translation_id, "Attempting to install translation plugin");
    out_info!("{}", theme::muted(format!("Installing {} translation plugin...", lang)));
    mark_translation_checked(&plugins_dir, &translation_id);

    let manager = PluginManager::new();
    if manager.install_plugin(&translation_id, None).await.is_ok() {
        tracing::trace!("Translation plugin installed, loading FTL");
        try_load_ftl(i18n, lang, &plugin_dir);
    } else {
        tracing::trace!("Translation plugin not available");
        out_warn!("Translation plugin {} not available, using English", translation_id);
    }
}

fn try_load_ftl(i18n: &mut I18n, lang: &str, plugin_dir: &std::path::Path) -> bool {
    let Some(ftl_path) = find_messages_ftl(plugin_dir) else {
        tracing::trace!("No FTL file found for translation plugin");
        return false;
    };
    tracing::trace!(path = %ftl_path.display(), "Found FTL file");

    match std::fs::read_to_string(&ftl_path) {
        Ok(content) => {
            let ok = i18n.load_embedded(lang, &content).is_ok();
            tracing::trace!(loaded = ok, "Loaded translation FTL");
            ok
        }
        Err(_) => {
            tracing::trace!("Failed to read FTL file");
            false
        }
    }
}

async fn get_available_languages() -> Vec<(String, String)> {
    tracing::trace!("Discovering available languages");
    let base = vec![("en-US".to_string(), "English".to_string())];

    let manager = PluginManager::new();
    if let Ok(plugins) = manager.list_plugins().await {
        let extra = registry_languages(&plugins);
        let languages = base.into_iter().chain(extra).collect::<Vec<_>>();
        tracing::trace!(count = languages.len(), "Available languages discovered");
        return languages;
    }

    tracing::trace!("Registry unreachable, scanning installed plugins for translations");
    let extra = installed_languages().await;
    let languages = base.into_iter().chain(extra).collect::<Vec<_>>();
    tracing::trace!(count = languages.len(), "Available languages discovered");
    languages
}

fn registry_languages(plugins: &[registry_client::PluginEntry]) -> Vec<(String, String)> {
    plugins
        .iter()
        .filter(|p| p.plugin_types.iter().any(|t| t == "translation"))
        .filter_map(|p| {
            let lang_code = p.id.strip_prefix(cli::clienv::CLI_PLUGIN_PREFIX)?;
            if lang_code == "en-US" {
                return None;
            }
            let display_name = p.name.strip_prefix("ADI CLI - ").unwrap_or(&p.name).to_string();
            tracing::trace!(lang = %lang_code, name = %display_name, "Found translation plugin in registry");
            Some((lang_code.to_string(), display_name))
        })
        .collect()
}

async fn installed_languages() -> Vec<(String, String)> {
    let plugins_dir = lib_plugin_host::PluginConfig::default_plugins_dir();
    let Ok(mut entries) = tokio::fs::read_dir(&plugins_dir).await else { return Vec::new() };

    let mut languages = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        let Some(lang_code) = name.strip_prefix(cli::clienv::CLI_PLUGIN_PREFIX) else { continue };
        if lang_code == "en-US" {
            continue;
        }
        let display_name = read_language_name_from_manifest(&entry.path()).await
            .unwrap_or_else(|| lang_code.to_string());
        tracing::trace!(lang = %lang_code, name = %display_name, "Found installed translation plugin");
        languages.push((lang_code.to_string(), display_name));
    }
    languages
}

async fn read_language_name_from_manifest(plugin_dir: &std::path::Path) -> Option<String> {
    let version = tokio::fs::read_to_string(plugin_dir.join(".version")).await.ok()?;
    let manifest_path = plugin_dir.join(version.trim()).join("plugin.toml");
    let content = std::fs::read_to_string(&manifest_path).ok()?;
    let table: toml::Table = content.parse().ok()?;
    table
        .get("translation")
        .and_then(|t| t.get("language_name"))
        .and_then(|n| n.as_str())
        .map(String::from)
}

async fn prompt_language_selection() -> anyhow::Result<String> {
    let languages = get_available_languages().await;

    if languages.len() <= 1 {
        tracing::trace!("Only en-US available, skipping language prompt");
        return Ok("en-US".to_string());
    }

    out_info!("{}", theme::brand_bold("Welcome to ADI! 🎉"));

    let items: Vec<(String, String)> = languages
        .iter()
        .map(|(code, name)| (format!("{} ({})", name, code), code.clone()))
        .collect();

    let selected = Select::new("Please select your preferred language:")
        .items(items)
        .run()
        .ok_or_else(|| anyhow::anyhow!("Language selection cancelled"))?;

    tracing::trace!(selected = %selected, "User selected language");
    Ok(selected)
}

fn should_check_translation(plugins_dir: &std::path::Path, translation_id: &str) -> bool {
    let stamp = plugins_dir.join(format!(".{}.last-check", translation_id));
    let should = match std::fs::metadata(&stamp) {
        Ok(meta) => meta
            .modified()
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_none_or(|age| age > std::time::Duration::from_secs(86400)),
        Err(_) => true,
    };
    tracing::trace!(translation_id = %translation_id, should_check = should, "Translation check status");
    should
}

fn mark_translation_checked(plugins_dir: &std::path::Path, translation_id: &str) {
    let stamp = plugins_dir.join(format!(".{}.last-check", translation_id));
    let _ = std::fs::create_dir_all(plugins_dir);
    let _ = std::fs::write(&stamp, []);
    tracing::trace!(translation_id = %translation_id, "Marked translation check timestamp");
}

fn find_messages_ftl(plugin_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    tracing::trace!(dir = %plugin_dir.display(), "Searching for messages.ftl");
    let result = find_versioned_ftl(plugin_dir)
        .or_else(|| find_direct_ftl(plugin_dir))
        .or_else(|| find_subdirectory_ftl(plugin_dir));
    if result.is_none() {
        tracing::trace!("No messages.ftl found");
    }
    result
}

fn find_versioned_ftl(plugin_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let version = std::fs::read_to_string(plugin_dir.join(".version")).ok()?;
    let ftl_path = plugin_dir.join(version.trim()).join("messages.ftl");
    if ftl_path.exists() {
        tracing::trace!(path = %ftl_path.display(), "Found versioned messages.ftl");
        Some(ftl_path)
    } else {
        None
    }
}

fn find_direct_ftl(plugin_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let ftl_path = plugin_dir.join("messages.ftl");
    if ftl_path.exists() {
        tracing::trace!(path = %ftl_path.display(), "Found direct messages.ftl");
        Some(ftl_path)
    } else {
        None
    }
}

fn find_subdirectory_ftl(plugin_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    std::fs::read_dir(plugin_dir).ok()?.flatten().find_map(|entry| {
        let subdir = entry.path();
        if !subdir.is_dir() {
            return None;
        }
        let ftl_path = subdir.join("messages.ftl");
        if ftl_path.exists() {
            tracing::trace!(path = %ftl_path.display(), "Found messages.ftl in subdirectory");
            Some(ftl_path)
        } else {
            None
        }
    })
}
