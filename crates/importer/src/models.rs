use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

pub mod flexible_time {
    use serde::{Deserialize, Deserializer};
    use serde_json::Value;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::Number(n) => Ok(n.as_f64()),
            Value::Null => Ok(None),
            _ => Ok(None),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Conversation {
    pub title: String,
    pub create_time: f64,
    pub update_time: f64,
    pub mapping: HashMap<String, MappingNode>,
    #[serde(default)]
    pub current_node: Option<String>,
    #[serde(default)]
    pub model_slug: Option<String>,
    #[serde(default)]
    pub is_archived: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct MappingNode {
    pub id: String,
    pub message: Option<Message>,
    pub parent: Option<String>,
    pub children: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub id: String,
    pub author: Author,
    #[serde(deserialize_with = "flexible_time::deserialize")]
    pub create_time: Option<f64>,
    pub update_time: Option<f64>,
    pub content: Content,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub end_turn: Option<bool>,
    #[serde(default)]
    pub weight: Option<f64>,
    #[serde(default)]
    pub metadata: Option<Value>,
    #[serde(default)]
    pub recipient: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Author {
    pub role: String,
    pub name: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct Content {
    pub content_type: String,
    #[serde(default)]
    pub parts: Vec<Value>,
    #[serde(default)]
    pub user_profile: Option<String>,
    #[serde(default)]
    pub user_instructions: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetPointer {
    pub asset_pointer: String,
    pub content_type: String,
    #[serde(default)]
    pub size_bytes: Option<i64>,
    #[serde(default)]
    pub width: Option<i32>,
    #[serde(default)]
    pub height: Option<i32>,
    #[serde(default)]
    pub metadata: Option<Value>,
}
