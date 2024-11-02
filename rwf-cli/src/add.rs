use rwf::macros::context;
use rwf::view::Template;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::logging::{created, error};

pub async fn controller(name: &str, page: bool, overwrite: bool) {
    match controller_internal(name, page, overwrite).await {
        Ok(_) => (),
        Err(_err) => {
            error("failed to create controller, did you run rwf-cli setup?");
        }
    }
}

async fn controller_internal(name: &str, page: bool, overwrite: bool) -> Result<(), rwf::Error> {
    let snake = rwf::snake_case(name);
    let ctx = context!("name" => rwf::pascal_case(&snake));
    let tpl = if page {
        Template::from_str(include_str!("templates/page-controller.rs.tpl"))?
    } else {
        Template::from_str(include_str!("templates/controller.rs.tpl"))?
    };
    let rendered = tpl.render(&ctx)?;

    let path = Path::new("src/controllers").join(format!("{}.rs", snake));

    if path.exists() && !overwrite {
        error(format!(
            "{} already exists, pass --overwrite to recreate it",
            snake,
        ));
        return Ok(());
    }

    let mut file = File::create(&path).await?;
    file.write_all(rendered.as_bytes()).await?;

    created(path.display().to_string());
    Ok(())
}
