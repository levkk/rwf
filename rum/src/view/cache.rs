use super::Template;
use std::collections::HashMap;

pub struct TemplateCache {
    templates: HashMap<String, Template>,
}

impl TemplateCache {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub async fn get(&self, name: &str) -> Option<&Template> {
        if let Some(template) = self.templates.get(name) {
            Some(template)
        } else {
            todo!()
        }
    }
}
