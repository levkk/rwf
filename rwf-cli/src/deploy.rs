use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use flate2::write::GzEncoder;
use flate2::Compression;
use rwf::config::ConfigFile;
use tokio::process::Command;

use crate::logging::*;
use crate::util::*;

pub async fn build() -> Result<bool, Box<dyn std::error::Error + 'static>> {
    let build = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .await?;

    if !build.success() {
        error("couldn't build the application, check build logs for error");
        return Ok(false);
    }

    Ok(true)
}

pub async fn package(config: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error + 'static>> {
    if !build().await? {
        return Ok(());
    }

    let archive = File::create("build.tar.gz")?;
    let enc = GzEncoder::new(archive, Compression::default());
    let mut tar = tar::Builder::new(enc);

    let info = package_info().await?;
    let executable = Path::new(&info.target_dir).join("release").join(&info.name);

    packaging("binary");
    tar.append_file(&info.name, &mut File::open(executable).expect("binary"))
        .expect("binary");

    for path in ["static", "templates", "migrations"] {
        let p = Path::new(path);

        if p.is_dir() {
            packaging(path);
            tar.append_dir_all(path, p)?;
        }
    }

    if let Some(config) = config {
        if config.is_file() {
            if let Err(_) = ConfigFile::load(&config) {
                warning(format!(
                    "{} doesn't seem to be be valid Rwf config file, but we'll use it anyway",
                    config.display()
                ));
            }
            packaging(config.display());
            tar.append_file("rwf.toml", &mut File::open(config)?)?;
        } else {
            warning(format!("{} does not exist, skipping", config.display()));
        }
    }

    created("build.tar.gz");

    Ok(())
}
