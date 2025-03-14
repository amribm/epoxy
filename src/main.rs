use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use io::Result;

use serde::Deserialize;

fn main() -> Result<()> {
    let path = PathBuf::from("config.json");
    let mut file = fs::File::open(path)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;

    let config: ProxyConfig = serde_json::from_str(&file_content).unwrap();
    println!("{:?}", config);

    Ok(())
}

struct EpoxyService {
    config: ProxyConfig,
    app_config_list: Vec<AppConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ProxyConfig {
    apps: Vec<AppConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AppConfig {
    name: String,
    ports: Vec<u16>,
    targets: Vec<String>,
}
