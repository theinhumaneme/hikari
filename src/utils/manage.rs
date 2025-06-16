use std::collections::HashMap;

use log::{error, info};

use crate::{
    objects::structs::{DeployConfig, HikariConfig, StackConfig},
    utils::docker_utils::{generate_compose, pull_compose, start_compose, stop_compose},
};

pub fn manage_node(
    current_config: &HikariConfig,
    incoming_config: &HikariConfig,
    client: &str,
    environment: &str,
    solution: &str,
) {
    for (key, current_deploy_config) in &current_config.deploy_configs {
        // filter current config matches the node's parameters
        if current_deploy_config.client != client
            || current_deploy_config.environment != environment
            || current_deploy_config.solution != solution
        {
            info!("Skipping config '{key}' as it does not match the node parameters.");
            continue;
        }
        // get the incoming config w.r.t to current config by key
        if let Some(incoming_deploy_config) = incoming_config.deploy_configs.get(key) {
            // Compare `current_config` and `incoming_config` values
            if current_deploy_config.client != incoming_deploy_config.client
                || current_deploy_config.environment != incoming_deploy_config.environment
                || current_deploy_config.solution != incoming_deploy_config.solution
            {
                // Values differ but still match the node; restart stacks
                info!(
                    "Config '{key}' parameters have changed, config no longer matches the node, Stopping associated stacks..."
                );
                current_deploy_config
                    .deploy_stacks
                    .iter()
                    .for_each(|stack| {
                        info!("Stopping Stack {}", stack.stack_name);
                        manage_stack(stack, "stop");
                    });
            } else {
                // Parameters match; compare stacks
                compare_stacks(current_deploy_config, incoming_deploy_config);
            }
        } else {
            // Config is removed (not in incoming_config)
            info!("Config '{key}' has been removed. Stopping associated stacks...");
            current_deploy_config
                .deploy_stacks
                .iter()
                .for_each(|stack| {
                    info!("Stopping Stack {}", stack.stack_name);
                    manage_stack(stack, "stop");
                });
        }
    }

    // Handle new deploy configs in incoming_config
    for (key, incoming_deploy_config) in &incoming_config.deploy_configs {
        // Skip configs that do not match the node's parameters
        if incoming_deploy_config.client != client
            || incoming_deploy_config.environment != environment
            || incoming_deploy_config.solution != solution
        {
            info!("Skipping new config '{key}' as it does not match the node parameters.");
            continue;
        }

        // Check if the config exists in current_config
        if let Some(current_deploy_config) = current_config.deploy_configs.get(key) {
            // Detect changes in client/environment/solution
            if incoming_deploy_config.client != *current_deploy_config.client
                || incoming_deploy_config.environment != *current_deploy_config.environment
                || incoming_deploy_config.solution != *current_deploy_config.solution
            {
                info!(
                    "Changes detected in config '{key}'. client/environment/solution now match the node parameters, Starting associated stacks..."
                );
                incoming_deploy_config
                    .deploy_stacks
                    .iter()
                    .for_each(|stack| {
                        info!("Starting Stack {}", stack.stack_name);
                        manage_stack(stack, "start");
                    });
            } else {
                // No changes detected
                info!("No changes detected for config '{key}'. Skipping.");
            }
        } else {
            // Handle new deploy configurations
            info!("New Deploy Config '{key}' found. Starting associated stacks...");
            incoming_deploy_config
                .deploy_stacks
                .iter()
                .for_each(|stack| {
                    manage_stack(stack, "start");
                });
        }
    }
}

fn compare_stacks(current_deploy_config: &DeployConfig, incoming_deploy_config: &DeployConfig) {
    // check if any stack has been deleted
    let current_stacks: HashMap<String, &StackConfig> = current_deploy_config
        .deploy_stacks
        .iter()
        .map(|stack| (stack.stack_name.clone(), stack))
        .collect();
    let incoming_stacks: HashMap<String, &StackConfig> = incoming_deploy_config
        .deploy_stacks
        .iter()
        .map(|stack| (stack.stack_name.clone(), stack))
        .collect();

    // first stop all the removed stacks
    let removed_stacks: Vec<&StackConfig> = current_stacks
        .keys()
        .filter(|key| !incoming_stacks.contains_key(*key))
        .filter_map(|key| current_stacks.get(key))
        .cloned()
        .collect();
    removed_stacks.iter().for_each(|stack| {
        info!("Stopping stack {}", stack.stack_name);
        let _ = manage_stack(stack, "stop");
    });
    for (stack_name, incoming_stack) in &incoming_stacks {
        if let Some(current_stack) = current_stacks.get(stack_name) {
            if current_stack == incoming_stack {
                info!("{} stack is unchanged", current_stack.stack_name);
                continue;
            } else {
                info!("changes detected in stack {}", current_stack.stack_name);
                info!("Stopping stack {}", current_stack.stack_name);
                match manage_stack(current_stack, "stop") {
                    true => {
                        info!("Starting Stack {}", incoming_stack.stack_name);
                        manage_stack(incoming_stack, "start");
                    }
                    false => continue,
                }
            }
        } else {
            manage_stack(incoming_stack, "start");
        }
    }
}

pub fn manage_stack(stack: &StackConfig, operation: &str) -> bool {
    match operation {
        "stop" => {
            match stop_compose(format!("{}/{}", stack.home_directory, stack.filename).as_str()) {
                true => {
                    info!("Successfully stopped removed stack {}", stack.stack_name);
                    true
                }
                false => {
                    error!("Could not stop removed stack {}", stack.stack_name);
                    false
                }
            }
        }
        "start" => {
            let stack_filepath: String = generate_compose(
                &stack.home_directory,
                &stack.stack_name,
                &stack.filename,
                &stack.compose_spec,
            );
            match pull_compose(&stack_filepath) {
                true => {
                    info!("Successfully pulled added stack {}", stack.stack_name);
                }
                false => {
                    error!("Could not pull added stack {}", stack.stack_name);
                    return false;
                }
            }

            match start_compose(&stack_filepath) {
                true => {
                    info!("Successfully started added stack {}", stack.stack_name);
                    true
                }
                false => {
                    error!("Could not start added stack {}", stack.stack_name);
                    false
                }
            }
        }
        _ => {
            error!("invalid operation {operation}");
            false
        }
    }
}
