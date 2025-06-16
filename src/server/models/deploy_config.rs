use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeployConfigDTO {
    pub id: Option<i64>,
    pub name: String,
    pub client: String,
    pub environment: String,
    pub solution: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_ids: Option<Vec<i64>>,
}
