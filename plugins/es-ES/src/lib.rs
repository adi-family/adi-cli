//! English translation plugin for ADI CLI (v3)
//!
//! Provides English (es-ES) translations via Fluent message format.

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

/// English translation plugin
pub struct Spanish(Spain)Plugin {
    metadata: TranslationMetadata,
}

impl Spanish(Spain)Plugin {
    pub fn new() -> Self {
        Self {
            metadata: TranslationMetadata {
                plugin_id: "adi.cli".to_string(),
                language: "es-ES".to_string(),
                language_name: "Spanish (Spain)".to_string(),
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

impl Plugin for Spanish(Spain)Plugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "adi.cli.es-ES".to_string(),
            name: "ADI CLI - English".to_string(),
            version: "3.0.0".to_string(),
            plugin_type: PluginType::Extension,
            author: Some("ADI Team".to_string()),
            description: Some("English translations for ADI CLI".to_string()),
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

impl Default for Spanish(Spain)Plugin {
    fn default() -> Self {
        Self::new()
    }
}

// Plugin entry point
#[no_mangle]
pub fn plugin_create() -> Box<dyn Plugin> {
    Box::new(Spanish(Spain)Plugin::new())
}
