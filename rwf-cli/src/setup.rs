use std::path::Path;
use tokio::fs::{create_dir_all, read_to_string, File};
use tokio::process::Command;

use crate::logging::created;
use rwf::colors::MaybeColorize;

pub async fn setup() {
    for dir in [
        "migrations",
        "templates",
        "src/controllers",
        "src/models",
        "static",
    ] {
        let path = Path::new(dir);

        if !path.exists() {
            create_dir_all(&path).await.expect("failed to create dir");
            created(format!("\"{}\" directory", path.display()));

            if path.starts_with("src") {
                let path = path.join("mod.rs");
                File::create(&path).await.expect("failed to create file");
                created(format!("\"{}\"", path.display()));
            } else {
                let git_keep = path.join(".gitkeep");
                File::create(&git_keep)
                    .await
                    .expect("failed to create .gitkeep");
                created(format!("\"{}\"", git_keep.display()));
            }
        }
    }

    for file in ["src/lib.rs", "src/main.rs"] {
        let path = Path::new(file);
        if path.exists() {
            let src = read_to_string(path).await.expect("cannot read source file");
            let (mut have_controllers, mut have_models) = (false, false);

            for line in src.lines() {
                if line.trim() == "mod controllers;" {
                    have_controllers = true;
                }

                if line.trim() == "mod models;" {
                    have_models = true;
                }
            }

            let mut src = String::new();

            if !have_controllers {
                src.push_str(&"mod".purple());
                src.push_str(" controllers;\n");
            }

            if !have_models {
                src.push_str(&"mod".purple());
                src.push_str(" models;\n");
            }

            if !src.is_empty() {
                eprintln!("Add the following code to \"{}\":\n", path.display());
                eprintln!("{}", src);
            }

            break;
        }
    }

    // Add rwf dependencies
    Command::new("cargo")
        .arg("add")
        .arg("rwf")
        .status()
        .await
        .unwrap();
}
