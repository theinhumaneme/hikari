use std::{
    fs::{self, File},
    io::copy,
};

use reqwest::{Error, blocking::Client};

use crate::objects::structs::HikariConfig;

pub fn download_file(file_url: &str, filename: &str) -> bool {
    let client = Client::builder().build().unwrap();
    let response = client.get(file_url).send().unwrap();
    if !response.status().is_success() {
        dbg!(response);
        return false;
    }
    let mut file = File::create(filename).unwrap();
    copy(&mut response.bytes().unwrap().as_ref(), &mut file).unwrap();
    true
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
