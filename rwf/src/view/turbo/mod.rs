use once_cell::sync::Lazy;

use super::{Context, Template};
use crate::view::template::lexer::value::ToTemplateValue;

static TEMPLATE: Lazy<Template> =
    Lazy::new(|| Template::from_str(include_str!("stream.html")).unwrap());

#[derive(Debug, Clone)]
pub struct TurboStream {
    action: String,
    template: String,
    target: String,
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
    pub fn new(template: impl ToString) -> Self {
        Self {
            action: "replace".into(),
            template: template.to_string(),
            target: "".into(),
        }
    }

    pub fn action(mut self, action: impl ToString) -> Self {
        self.action = action.to_string();
        self
    }

    pub fn target(mut self, target: impl ToString) -> Self {
        self.target = target.to_string();
        self
    }

    pub fn render(self) -> String {
        let context: Context = self.into();
        TEMPLATE.render(&context).unwrap()
    }
}

pub struct TurboStreams {
    streams: Vec<TurboStream>,
}

impl TurboStreams {
    pub fn new() -> Self {
        Self { streams: vec![] }
    }

    pub fn add(&mut self, stream: TurboStream) {
        self.streams.push(stream);
    }

    pub fn render(self) -> String {
        let mut results = vec![];
        for stream in self.streams {
            results.push(stream.render());
        }
        results.join("\n")
    }
}
