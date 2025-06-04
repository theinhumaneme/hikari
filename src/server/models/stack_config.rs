use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::container::ContainerDTO;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackConfigDTO {
    pub id: i64,
    pub deployment_id: i64,
    pub stack_name: String,
    pub filename: String,
    pub home_directory: String,
    pub compose_spec: ComposeSpecDTO,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComposeSpecDTO {
    pub services: HashMap<String, ContainerDTO>,
}
