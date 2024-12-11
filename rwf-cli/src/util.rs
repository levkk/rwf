use tokio::fs::read_to_string;
use tokio::process::Command;
use toml::Value;

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    #[allow(dead_code)]
    pub version: String,
    pub target_dir: String,
    pub rwf_auth: bool,
}

async fn cargo_toml() -> Result<Value, Box<dyn std::error::Error + 'static>> {
    let cargo_toml = read_to_string("Cargo.toml").await?;
    let toml: Value = toml::from_str(&cargo_toml)?;

    Ok(toml)
}

pub async fn package_info() -> Result<PackageInfo, Box<dyn std::error::Error + 'static>> {
    let toml = cargo_toml().await?;

    let name = toml
        .get("package")
        .expect("Cargo.toml to have a valid [package] attribute")
        .get("name")
        .expect("Cargo.toml to have a valid \"name\" field");

    let version = toml
        .get("package")
        .expect("Cargo.toml to have a valid [package] attribute")
        .get("version")
        .expect("Cargo.toml to have a valid \"name\" field");

    let rwf_auth = toml
        .get("dependencies")
        .expect("Cargo.toml to have a valid [dependencies] attribute");

    let rwf_auth = rwf_auth
        .as_table()
        .expect("[dependencies] to be a table")
        .iter()
        .any(|dep| dep.0 == "rwf-auth");

    let metadata = Command::new("cargo").arg("metadata").output().await?.stdout;
    let json: serde_json::Value = serde_json::from_slice(&metadata)?;
    let target_dir = json["target_directory"].as_str().unwrap().to_string();

    Ok(PackageInfo {
        name: name.as_str().unwrap().to_string(),
        version: version.as_str().unwrap().to_string(),
        target_dir,
        rwf_auth,
    })
}
