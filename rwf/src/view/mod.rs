//! Dynamic templates and views, the **V** in MVC.
//!
//! Rwf templates are inspired from multiple other languages like ERB and Jinja2. The templates are dynamic,
//! meaning they are interpreted at runtime. This allows for smoother local development experience, while the
//! template cache in production makes this performant as well.
//!
//! # Example
//!
//! ```
//! # use rwf::view::*;
//! let template = Template::from_str("<h1><%= title %></h1>").unwrap();
//! let mut context = Context::new();
//!
//! context.set("title", "Hello from Rwf!").unwrap();
//!
//! let rendered = template.render(&context).unwrap();
//!
//! assert_eq!(rendered, "<h1>Hello from Rwf!</h1>");
//! ```
//!
//! # User guides
//!
//! See [documentation](https://levkk.github.io/rwf/views/) on how to use templates.
pub mod cache;
pub mod prelude;
pub mod template;
pub mod turbo;

pub use cache::Templates;
pub use template::Context;
pub use template::Error;
pub use template::Template;
pub use turbo::TurboStream;

pub use template::{ToTemplateValue, Value};
