use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackConfigDTO {
    pub id: Option<i64>,
    pub deployment_id: i64,
    pub stack_name: String,
    pub filename: String,
    pub home_directory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub containers: Option<Vec<i64>>,
}
