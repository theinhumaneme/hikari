use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};

use crate::objects::structs::ComposeSpec;
pub fn generate_compose(
    compose_directory: String,
    stack_name: String,
    filename: String,
    compose_config: ComposeSpec,
) -> String {
    let yaml = serde_yaml::to_string(&compose_config).unwrap();
    if !Path::new(&compose_directory).exists() {
        // Create the folder if it doesn't exist
        create_dir_all(format!("{}", compose_directory.clone())).unwrap();
        println!("Directory created:{}", compose_directory);
    } else {
        println!("Directory already exists: {}", compose_directory);
    }
    let base_path = format!("{}/{}", compose_directory, filename)
        .to_string()
        .to_owned();
    let mut file = File::create(base_path.clone()).unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    println!("Generating Compose for {} Complete", stack_name);
    base_path
}
