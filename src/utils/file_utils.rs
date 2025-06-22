use log::error;
use reqwest::Error;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use crate::objects::structs::HikariConfig;

pub async fn download_file(file_url: &str, filename: &str) -> Result<bool, Error> {
    let client = reqwest::Client::new();
    let mut response = client.get(file_url).send().await?;
    if !response.status().is_success() {
        return Ok(false);
    }
    let mut file = File::create(filename).await.unwrap();
    while let Some(chunk) = response.chunk().await? {
        if let Err(err) = file.write_all(&chunk).await {
            error!("Unable to download the file - Error {err}");
        }
    }
    Ok(true)
}

pub async fn copy_file(source: &str, destination: &str) {
    if let Ok(contents) = fs::read(source).await {
        let _ = fs::write(destination, contents).await;
    }
}
pub async fn write_file(contents: &str, destination: &str) -> std::io::Result<()> {
    fs::write(destination, contents).await
}
pub async fn load_config_from_url(url: &str) -> Result<HikariConfig, Error> {
    let response = reqwest::get(url).await?;
    let cfg = response.json::<HikariConfig>().await?;
    Ok(cfg)
}
