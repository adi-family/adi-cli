use lib_plugin_abi_v3::*;

#[allow(dead_code)]
const MESSAGES_FTL: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/messages.ftl"));

struct TranslationPlugin;

#[async_trait]
impl Plugin for TranslationPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(
            env!("TRANSLATION_ID"),
            env!("TRANSLATION_DISPLAY_NAME"),
            env!("CARGO_PKG_VERSION"),
        )
        .with_author("ADI Team")
        .with_description(format!(
            "{} translations for ADI CLI",
            env!("TRANSLATION_LANG_NAME"),
        ))
    }

    async fn init(&mut self, _ctx: &PluginContext) -> Result<()> {
        Ok(())
    }
}

#[no_mangle]
pub fn plugin_create() -> Box<dyn Plugin> {
    Box::new(TranslationPlugin)
}
