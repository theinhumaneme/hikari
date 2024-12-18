use std::{
    fs::{self, File},
    io::copy,
};

use reqwest::blocking::Client;

pub fn download_file(file_url: &str, filename: &str) -> bool {
    let client = Client::builder().build().unwrap();
    let response = client.get(file_url).send().unwrap();
    if !response.status().is_success() {
        dbg!(response);
        return false;
    }
    let mut file = File::create(filename).unwrap();
    copy(&mut response.bytes().unwrap().as_ref(), &mut file).unwrap();
    return true;
}

pub fn copy_file(source: &str, destination: &str) {
    let _ = fs::write(destination, fs::read(source).unwrap());
}
