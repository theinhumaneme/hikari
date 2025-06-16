use std::{
    fs::{File, create_dir_all},
    io::{self, BufRead, Write},
    path::Path,
    process::{Command, Stdio},
    thread,
};

use log::{error, info};

use crate::objects::structs::ComposeSpec;
pub fn dry_run_generate_compose(filename: String, compose_config: ComposeSpec) {
    let yaml = serde_yaml::to_string(&compose_config).unwrap();
    let base_path = format!("./{filename}").to_string().to_owned();
    let mut file = File::create(&base_path).unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
}
pub fn generate_compose(
    compose_directory: &str,
    stack_name: &str,
    filename: &str,
    compose_config: &ComposeSpec,
) -> String {
    let yaml = serde_yaml::to_string(&compose_config).unwrap();
    if !Path::new(&compose_directory).exists() {
        // Create the folder if it doesn't exist
        create_dir_all(compose_directory).unwrap();
        info!("Directory created:{compose_directory}");
    } else {
        info!("Directory already exists: {compose_directory}");
    }
    let base_path = format!("{compose_directory}/{filename}")
        .to_string()
        .to_owned();
    let mut file = File::create(&base_path).unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    info!("Generating Compose for {stack_name} Complete");
    base_path
}

pub fn execute_command(command: &str, args: Vec<&str>) -> bool {
    match Command::new(command)
        .args(&args)
        .stdin(Stdio::null()) // No input needed
        .stdout(Stdio::piped()) // Capture output
        .stderr(Stdio::piped()) // Capture error output
        .spawn()
    {
        Ok(mut child) => {
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            // Thread to handle stdout
            let stdout_thread = if let Some(stdout) = stdout {
                let reader = io::BufReader::new(stdout);
                thread::spawn(move || {
                    for line in reader.lines() {
                        match line {
                            Ok(line) => info!("{line}"), // Print each line of stdout
                            Err(e) => error!("Error reading stdout: {e}"),
                        }
                    }
                })
            } else {
                thread::spawn(|| {})
            };

            // Thread to handle stderr
            let stderr_thread = if let Some(stderr) = stderr {
                let reader = io::BufReader::new(stderr);
                thread::spawn(move || {
                    for line in reader.lines() {
                        match line {
                            Ok(line) => error!("ERROR: {line}"), // Print each line of stderr
                            Err(e) => error!("Error reading stderr: {e}"),
                        }
                    }
                })
            } else {
                thread::spawn(|| {})
            };

            // Wait for the process to finish
            match child.wait() {
                Ok(status) => {
                    let _ = stdout_thread.join();
                    let _ = stderr_thread.join();

                    if status.success() {
                        true
                    } else {
                        error!("Command exited with status: {status}");
                        false
                    }
                }
                Err(e) => {
                    error!("Failed to wait for the command: {e}");
                    false
                }
            }
        }
        Err(e) => {
            error!("Failed to execute command '{command}': {e}");
            false
        }
    }
}

pub fn pull_compose(compose_file_path: &str) -> bool {
    info!("{}", &compose_file_path);
    let command = "docker";
    let args = ["compose", "-f", compose_file_path, "pull"];
    if Path::exists(Path::new(compose_file_path)) {
        return execute_command(command, args.to_vec());
    }
    error!("compose file does not exist");
    false
}

pub fn start_compose(compose_file_path: &str) -> bool {
    info!("{}", &compose_file_path);
    let command = "docker";
    let args = ["compose", "-f", compose_file_path, "up", "-d"];
    if Path::exists(Path::new(compose_file_path)) {
        return execute_command(command, args.to_vec());
    }
    error!("compose file does not exist");
    false
}
pub fn stop_compose(compose_file_path: &str) -> bool {
    info!("{}", &compose_file_path);
    let command = "docker";
    let args = ["compose", "-f", compose_file_path, "down"];
    if Path::exists(Path::new(compose_file_path)) {
        return execute_command(command, args.to_vec());
    }
    error!("compose file does not exist");
    false
}
