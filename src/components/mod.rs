mod base_component;
mod base_tasks_component;
mod indexer_cli;
mod indexer_mcp;
mod llm_code_indexer_http;
mod tasks_cli;
mod tasks_http;
mod tasks_mcp;

pub use indexer_cli::IndexerCli;
pub use indexer_mcp::IndexerMcp;
pub use llm_code_indexer_http::LlmCodeIndexerHttp;
pub use tasks_cli::TasksCli;
pub use tasks_http::TasksHttp;
pub use tasks_mcp::TasksMcp;

use crate::registry::ComponentRegistry;

/// Creates the default component registry with all ADI components
pub fn create_default_registry() -> ComponentRegistry {
    let mut registry = ComponentRegistry::new();

    // Indexer components
    registry.register(Box::new(IndexerCli::new()));
    registry.register(Box::new(IndexerMcp::new()));
    registry.register(Box::new(LlmCodeIndexerHttp::new()));

    // Tasks components
    registry.register(Box::new(TasksCli::new()));
    registry.register(Box::new(TasksHttp::new()));
    registry.register(Box::new(TasksMcp::new()));

    registry
}
