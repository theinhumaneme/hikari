use std::{
    fs::{File, create_dir_all},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
};

use log::{error, info};

use crate::objects::structs::ComposeSpec;

pub fn dry_run_generate_compose(
    filename: String,
    compose_config: ComposeSpec,
) -> Result<PathBuf, io::Error> {
    let yaml = serde_yaml::to_string(&compose_config)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let base_path = PathBuf::from(format!("./{filename}"));
    let mut file = File::create(&base_path)?;
    file.write_all(yaml.as_bytes())?;
    Ok(base_path)
}
pub fn generate_compose(
    compose_directory: &str,
    stack_name: &str,
    filename: &str,
    compose_config: &ComposeSpec,
) -> Result<PathBuf, io::Error> {
    let yaml = serde_yaml::to_string(compose_config)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    if !Path::new(compose_directory).exists() {
        create_dir_all(compose_directory)?;
        info!("Directory created:{compose_directory}");
    } else {
        info!("Directory already exists: {compose_directory}");
    }
    let base_path = Path::new(compose_directory).join(filename);
    let mut file = File::create(&base_path)?;
    file.write_all(yaml.as_bytes())?;
    info!("Generating Compose for {stack_name} Complete");
    Ok(base_path)
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
            let stdout_thread: Option<thread::JoinHandle<()>> = stdout.map(|stdout| {
                let reader = io::BufReader::new(stdout);
                thread::spawn(move || {
                    for line in reader.lines() {
                        match line {
                            Ok(line) => info!("{line}"), // Print each line of stdout
                            Err(e) => error!("Error reading stdout: {e}"),
                        }
                    }
                })
            });

            // Thread to handle stderr
            let stderr_thread: Option<thread::JoinHandle<()>> = stderr.map(|stderr| {
                let reader = io::BufReader::new(stderr);
                thread::spawn(move || {
                    for line in reader.lines() {
                        match line {
                            Ok(line) => error!("ERROR: {line}"), // Print each line of stderr
                            Err(e) => error!("Error reading stderr: {e}"),
                        }
                    }
                })
            });

            // Wait for the process to finish
            match child.wait() {
                Ok(status) => {
                    if let Some(handle) = stdout_thread {
                        let _ = handle.join();
                    }
                    if let Some(handle) = stderr_thread {
                        let _ = handle.join();
                    }

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
