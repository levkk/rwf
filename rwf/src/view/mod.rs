//! Dynamic templates and views, the **V** in MVC.
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
