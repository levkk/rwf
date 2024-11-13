//! Turbo Stream implementation on the backend.
//!
//! Turbo Streams are template partials that can dynamically replace
//! DOM elements, similarly to a single page application written with React or Vue.
use once_cell::sync::Lazy;

use super::{Context, Template};
use crate::view::template::lexer::value::ToTemplateValue;

static TEMPLATE: Lazy<Template> =
    Lazy::new(|| Template::from_str(include_str!("stream.html")).unwrap());

/// Turbo Stream.
///
/// Renders a `<template>` which will be used by
/// Turbo on the frontend to perform the requested action, e.g.
/// replace a DOM element.
#[derive(Debug, Clone)]
pub struct TurboStream {
    action: String,
    template: String,
    target: String,
}

impl std::fmt::Display for TurboStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone().render())
    }
}

impl From<TurboStream> for Context {
    fn from(stream: TurboStream) -> Context {
        let mut context = Context::new();
        context["action"] = stream.action.to_template_value().unwrap();
        context["template"] = stream.template.to_template_value().unwrap();
        context["target"] = stream.target.to_template_value().unwrap();

        context
    }
}

impl TurboStream {
    /// Create new Turbo Stream from a rendered template.
    ///
    /// By default, the action is to replace an existing target. The target
    /// needs to be set by calling [`TurboStream::target`].
    pub fn new(template: impl ToString) -> Self {
        Self {
            action: "replace".into(),
            template: template.to_string(),
            target: "".into(),
        }
    }

    /// Turbo Stream action, e.g. "replace", "append", or "remove".
    pub fn action(mut self, action: impl ToString) -> Self {
        self.action = action.to_string();
        self
    }

    /// Turbo Stream target, i.e. a unique ID of a DOM element.
    pub fn target(mut self, target: impl ToString) -> Self {
        self.target = target.to_string();
        self
    }

    /// Render the Turbo Stream `<template>`.
    pub fn render(self) -> String {
        let context: Context = self.into();
        TEMPLATE.render(&context).unwrap()
    }
}
