use rwf::macros::context;
use rwf::view::Template;
use rwf::Error;
use std::path::Path;
use tokio::fs::{read_dir, File};
use tokio::io::AsyncWriteExt;

use crate::logging::{created, error, written};

pub async fn modules(path: &Path) -> Result<(), Error> {
    if !path.is_dir() {
        error(format!("\"{}\" is not a directory", path.display()));
        return Ok(());
    }

    let mut modules = vec![];
    let mut entries = read_dir(path).await?;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        if name != "mod" {
            modules.push(name);
        }
    }

    modules.sort();

    let tpl = Template::from_str(include_str!("templates/mod.rs.tpl"))?;
    let rendered = tpl.render(&context!("modules" => modules))?;

    let path = path.join("mod.rs");

    let mut file = File::create(&path).await?;
    file.write_all(rendered.trim().as_bytes()).await?;

    written(path.display().to_string());
    Ok(())
}

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

    let mod_path = Path::new("src/controllers");
    let path = mod_path.join(format!("{}.rs", snake));

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

    modules(&mod_path).await?;
    Ok(())
}
