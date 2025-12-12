use std::collections::HashMap;

use crate::component::Component;
use crate::error::{InstallerError, Result};

pub struct ComponentRegistry {
    components: HashMap<String, Box<dyn Component>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn register(&mut self, component: Box<dyn Component>) {
        let name = component.info().name.clone();
        self.components.insert(name, component);
    }

    pub fn get(&self, name: &str) -> Result<&dyn Component> {
        self.components
            .get(name)
            .map(|c| c.as_ref())
            .ok_or_else(|| InstallerError::ComponentNotFound(name.to_string()))
    }

    pub fn list(&self) -> Vec<&dyn Component> {
        self.components.values().map(|c| c.as_ref()).collect()
    }

    pub fn names(&self) -> Vec<&str> {
        self.components.keys().map(|s| s.as_str()).collect()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.components.contains_key(name)
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
