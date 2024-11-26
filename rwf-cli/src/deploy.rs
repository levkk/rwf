use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use flate2::write::GzEncoder;
use flate2::Compression;
use rwf::config::get_config;
use rwf::config::Config;
use tokio::process::Command;

use crate::logging::*;
use crate::util::*;

pub async fn build(target: Option<String>) -> Result<bool, Box<dyn std::error::Error + 'static>> {
    let mut build = Command::new("cargo");

    build.arg("build").arg("--release");

    if let Some(target) = target {
        if !check_target(&target)? {
            return Ok(false);
        };

        build.arg("--target").arg(target);
    }

    let result = build.status().await?;

    if !result.success() {
        error("couldn't build the application, check build logs for error");
        return Ok(false);
    }

    Ok(true)
}

fn check_target(target: &str) -> Result<bool, Box<dyn std::error::Error + 'static>> {
    for path in [".cargo/config", ".cargo/config.toml"] {
        if let Ok(mut file) = File::open(path) {
            let mut config = String::new();
            file.read_to_string(&mut config)?;

            if let Ok(config) = toml::from_str::<toml::Value>(&config) {
                if let Some(target_conf) = config.get("target") {
                    if let Some(config) = target_conf.get(target) {
                        if let Some(linker) = config.get("linker") {
                            let linker = linker.as_str().unwrap();
                            if let Ok(linker_path) = which::which(linker) {
                                using(format!(
                                    "linker for target \"{}\" in {}",
                                    target,
                                    linker_path.display()
                                ));
                                return Ok(true);
                            } else {
                                error(format!(
                                    "target \"{}\" doesn't have the linker \"{}\" in $PATH",
                                    target, linker
                                ));
                                return Ok(false);
                            }
                        } else {
                            error(format!(
                                "target \"{}\" requires a linker configured in .cargo/config.toml",
                                target
                            ));
                            return Ok(false);
                        }
                    } else {
                        error(format!(
                            "target \"{}\" is not configured in .cargo/config.toml",
                            target,
                        ));
                        return Ok(false);
                    }
                } else {
                    error(format!(
                        "target \"{}\" is not configured in .cargo/config.toml",
                        target,
                    ));
                    return Ok(false);
                }
            }

            break;
        }
    }

    error(".cargo/config.toml doesn't exist or has invalid format");
    Ok(false)
}

pub async fn package(
    config: Option<PathBuf>,
    target: Option<String>,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    if !build(target.clone()).await? {
        return Ok(());
    }

    let archive = File::create("build.tar.gz")?;
    let enc = GzEncoder::new(archive, Compression::default());
    let mut tar = tar::Builder::new(enc);

    let info = package_info().await?;
    let executable = Path::new(&info.target_dir)
        .join(if let Some(target) = &target {
            Path::new(target).join("release").to_owned()
        } else {
            Path::new("release").to_owned()
        })
        .join(&info.name);

    let mut paths = get_config().package.include.clone();
    paths.extend(get_config().package.include_additional.clone());
    let paths = paths
        .into_iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>();

    packaging("executable");

    let bin_name = format!("{}.bin", info.name);

    if paths.contains(&bin_name) {
        error(format!(
            "asset directory \"{}\" has the same name as the binary executable and will be overwritten in the package",
            info.name
        ));
        return Ok(());
    }

    tar.append_file(&bin_name, &mut File::open(executable).expect("binary"))
        .expect("binary");

    for path in paths {
        let p = Path::new(&path);
        if p.is_dir() {
            packaging(&path);
            tar.append_dir_all(&path, p)?;
        }
    }

    if let Some(config) = config {
        if config.is_file() {
            if let Err(_) = Config::load(&config) {
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
