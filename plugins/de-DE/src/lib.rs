//! German translation plugin for ADI CLI (v3)
//!
//! Provides German (de-DE) translations via Fluent message format.

use lib_plugin_abi_v3::*;
use serde::{Deserialize, Serialize};

// Embedded Fluent messages at compile time
const MESSAGES_FTL: &str = include_str!("../messages.ftl");

/// Translation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TranslationMetadata {
    plugin_id: String,
    language: String,
    language_name: String,
    namespace: String,
    version: String,
}

/// German translation plugin
pub struct GermanPlugin {
    metadata: TranslationMetadata,
}

impl GermanPlugin {
    pub fn new() -> Self {
        Self {
            metadata: TranslationMetadata {
                plugin_id: "adi.cli".to_string(),
                language: "de-DE".to_string(),
                language_name: "German (Germany)".to_string(),
                namespace: "cli".to_string(),
                version: "3.0.0".to_string(),
            },
        }
    }

    /// Get Fluent messages (.ftl file content)
    pub fn get_messages(&self) -> &'static str {
        MESSAGES_FTL
    }

    /// Get translation metadata as JSON
    pub fn get_metadata_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self.metadata)?)
    }
}

#[async_trait]
impl Plugin for GermanPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "adi.cli.de-DE".to_string(),
            name: "ADI CLI - German".to_string(),
            version: "3.0.0".to_string(),
            plugin_type: PluginType::Extension,
            author: Some("ADI Team".to_string()),
            description: Some("German translations for ADI CLI".to_string()),
            category: None,
        }
    }

    async fn init(&mut self, _ctx: &PluginContext) -> Result<()> {
        // No initialization needed for static translations
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        // No cleanup needed
        Ok(())
    }
}

impl Default for GermanPlugin {
    fn default() -> Self {
        Self::new()
    }
}

// Plugin entry point
#[no_mangle]
pub fn plugin_create() -> Box<dyn Plugin> {
    Box::new(GermanPlugin::new())
}
