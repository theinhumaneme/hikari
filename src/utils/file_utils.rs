use std::{
    fs::{self, File},
    io::copy,
};

use log::error;
use reqwest::{Error, blocking::Client};

use crate::objects::structs::HikariConfig;

pub fn download_file(file_url: &str, filename: &str) -> Result<bool, Error> {
    let client = Client::builder().build()?;
    let response = client.get(file_url).send()?;
    if !response.status().is_success() {
        return Ok(false);
    }
    let mut file = File::create(filename).unwrap();
    let _ = copy(&mut response.bytes()?.as_ref(), &mut file)
        .map_err(|err| error!("Unable to download the file - Error {err}"));
    Ok(true)
}

pub fn copy_file(source: &str, destination: &str) {
    let _ = fs::write(destination, fs::read(source).unwrap());
}
pub fn write_file(contents: &str, destination: &str) {
    let _ = fs::write(destination, contents);
}
pub async fn load_config_from_url(url: &str) -> Result<HikariConfig, Error> {
    let response = reqwest::get(url).await?;
    let cfg = response.json::<HikariConfig>().await?;
    Ok(cfg)
}
