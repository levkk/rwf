use crate::add::modules;
use crate::logging::{error, removed};
use rwf::controller::Error;
use std::path::Path;
use tokio::fs::remove_file;

pub async fn controller(name: &str) -> Result<(), Error> {
    let snake = rwf::snake_case(name);
    let mod_path = Path::new("src/controllers");
    let path = mod_path.join(format!("{}.rs", snake));

    if path.exists() {
        remove_file(&path).await?;

        removed(path.display().to_string());

        modules(&mod_path).await?;
    } else {
        error(format!("{} doesn't exist", path.display()));
    }

    Ok(())
}
