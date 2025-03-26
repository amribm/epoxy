use std::{
    fs,
    io::{Read},
    path::PathBuf,
};

mod lib;

use lib::application::{AppError,Application,AppConfig};
use serde::Deserialize;



#[tokio::main]
async fn main() -> Result<(),AppError> {
    let path = PathBuf::from("config.json");
    let mut file = fs::File::open(path)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;

    let config: ProxyConfig = serde_json::from_str(&file_content).unwrap();

    let service = EpoxyService::new(config);

    service.start().await?;

    Ok(())
}

struct EpoxyService {
    config: ProxyConfig,
    app_config_list: Vec<AppConfig>,
    // runtime_handler: tokio::runtime::Handle,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ProxyConfig {
    apps: Vec<AppConfig>,
}


impl EpoxyService {
    fn new(config: ProxyConfig) -> EpoxyService {
        EpoxyService {
            app_config_list: config.apps.clone(),
            config: config,
        }
    }

    async fn start(&self) -> Result<(),AppError> {
        let mut handler_set = tokio::task::JoinSet::new();
        for app_config in self.app_config_list.iter() {
           let app = Application::try_from(app_config.clone())?;
            let app_handle =tokio::task::spawn(async move {
                if let Err(e) = app.start().await {
                    println!("error occured at start: {}",e)
                }
            });
            handler_set.spawn(app_handle);
        }

        handler_set.join_all().await;
        Ok(())
    }
}
