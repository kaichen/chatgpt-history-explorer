use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use zip::ZipArchive;

pub mod models;
pub use models::*;

pub fn extract_conversations_from_zip(zip_path: &PathBuf) -> Result<String> {
    let file = fs::File::open(zip_path)
        .with_context(|| format!("Failed to open zip file: {}", zip_path.display()))?;

    let mut archive = ZipArchive::new(file).context("Failed to read zip archive")?;

    let mut conversations_file = archive
        .by_name("conversations.json")
        .context("conversations.json not found in zip file")?;

    let mut contents = String::new();
    conversations_file
        .read_to_string(&mut contents)
        .context("Failed to read conversations.json from zip")?;

    Ok(contents)
}

pub fn create_database(db_path: &PathBuf) -> Result<Connection> {
    let conn = Connection::open(db_path)
        .with_context(|| format!("Failed to create database: {}", db_path.display()))?;

    let schema_sql = fs::read_to_string("schema.sql")
        .context("Failed to read schema.sql - make sure it exists in current directory")?;

    conn.execute_batch(&schema_sql)
        .context("Failed to execute schema.sql")?;

    Ok(conn)
}

pub fn import_conversations(
    conn: &Connection,
    conversations: &[Conversation],
    zip_path: &PathBuf,
) -> Result<()> {
    conn.execute("PRAGMA foreign_keys = OFF", [])?;
    let mut conv_count = 0;
    let mut msg_count = 0;
    let mut asset_count = 0;

    for conv in conversations {
        let conv_id = find_conversation_id(conv);

        conn.execute(
            "INSERT OR REPLACE INTO conversations (id, title, create_time, update_time, model_slug, is_archived) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                conv_id,
                conv.title,
                conv.create_time as i64,
                conv.update_time as i64,
                conv.model_slug,
                conv.is_archived.unwrap_or(false)
            ],
        )
        .with_context(|| format!("Failed to insert conversation: {}", conv.title))?;

        conv_count += 1;

        let messages = extract_messages_from_mapping(&conv_id, &conv.mapping);
        for (order, (msg_id, msg, parent_id)) in messages.iter().enumerate() {
            if should_skip_message(msg) {
                continue;
            }

            let (text_content, assets) = extract_content_and_assets(&msg.content);
            let has_assets = !assets.is_empty();

            conn.execute(
                "INSERT OR REPLACE INTO messages (id, conversation_id, parent_id, author_role, content_type, text_content, create_time, model_slug, message_order, has_assets) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    msg_id,
                    conv_id,
                    parent_id,
                    msg.author.role,
                    msg.content.content_type,
                    text_content,
                    msg.create_time.map(|t| t as i64),
                    extract_model_slug(msg, conv),
                    order as i32,
                    has_assets
                ],
            )
            .with_context(|| format!("Failed to insert message: {}", msg_id))?;

            msg_count += 1;

            for (asset_order, asset) in assets.iter().enumerate() {
                let asset_id = extract_asset_id(&asset.asset_pointer);
                let metadata_json = asset
                    .metadata
                    .as_ref()
                    .map(|m| serde_json::to_string(m).unwrap_or_default());

                let (file_content, file_name, mime_type) =
                    extract_file_from_zip(zip_path, &asset.asset_pointer)?;

                conn.execute(
                    "INSERT OR REPLACE INTO assets (id, message_id, asset_pointer, content_type, size_bytes, width, height, metadata, asset_order, file_content, file_name, mime_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    params![
                        asset_id,
                        msg_id,
                        asset.asset_pointer,
                        asset.content_type,
                        asset.size_bytes,
                        asset.width,
                        asset.height,
                        metadata_json,
                        asset_order as i32,
                        file_content,
                        file_name,
                        mime_type
                    ],
                )
                .with_context(|| format!("Failed to insert asset: {}", asset_id))?;

                asset_count += 1;
            }
        }
    }

    conn.execute("PRAGMA foreign_keys = ON", [])?;

    println!(
        "Imported {} conversations, {} messages, and {} assets",
        conv_count, msg_count, asset_count
    );
    Ok(())
}

fn extract_content_and_assets(content: &Content) -> (Option<String>, Vec<AssetPointer>) {
    let mut text_content = None;
    let mut assets = Vec::new();

    for part in &content.parts {
        match part {
            Value::String(text) => {
                text_content = Some(text.clone());
            }
            Value::Object(_) => {
                if let Ok(asset) = serde_json::from_value::<AssetPointer>(part.clone()) {
                    assets.push(asset);
                }
            }
            _ => {}
        }
    }

    (text_content, assets)
}

fn extract_asset_id(asset_pointer: &str) -> String {
    asset_pointer
        .split("file-")
        .last()
        .unwrap_or(asset_pointer)
        .to_string()
}

fn extract_file_from_zip(zip_path: &PathBuf, asset_pointer: &str) -> Result<(Vec<u8>, String, String)> {
    let file = fs::File::open(zip_path)
        .with_context(|| format!("Failed to open zip file: {}", zip_path.display()))?;

    let mut archive = ZipArchive::new(file).context("Failed to read zip archive")?;

    let asset_id = extract_asset_id(asset_pointer);

    for i in 0..archive.len() {
        let file_name = {
            let file = archive.by_index(i)?;
            file.name().to_string()
        };

        if file_name.contains(&asset_id) {
            let mut file = archive.by_index(i)?;
            let mut content = Vec::new();
            file.read_to_end(&mut content)
                .with_context(|| format!("Failed to read file: {}", file_name))?;

            let mime_type = guess_mime_type(&file_name);

            return Ok((content, file_name, mime_type));
        }
    }

    println!("Warning: File not found in zip for asset: {}", asset_pointer);
    Ok((Vec::new(), String::new(), String::new()))
}

fn guess_mime_type(file_name: &str) -> String {
    let extension = file_name.split('.').last().unwrap_or("").to_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "png" => "image/png".to_string(),
        "gif" => "image/gif".to_string(),
        "webp" => "image/webp".to_string(),
        "pdf" => "application/pdf".to_string(),
        "txt" => "text/plain".to_string(),
        "json" => "application/json".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

fn find_conversation_id(conv: &Conversation) -> String {
    for node in conv.mapping.values() {
        if let Some(msg) = &node.message {
            if msg.author.role == "user" || msg.author.role == "assistant" {
                if !msg.content.parts.is_empty() {
                    if let Value::String(s) = &msg.content.parts[0] {
                        if !s.is_empty() {
                            return format!("conv_{}", msg.id);
                        }
                    }
                }
            }
        }
    }

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    conv.title.hash(&mut hasher);
    conv.create_time.to_bits().hash(&mut hasher);
    format!("conv_{:x}", hasher.finish())
}

fn extract_messages_from_mapping(
    _conv_id: &str,
    mapping: &HashMap<String, MappingNode>,
) -> Vec<(String, Message, Option<String>)> {
    let mut messages = Vec::new();

    let mut visited = std::collections::HashSet::new();

    let root_node = mapping
        .values()
        .find(|node| {
            node.parent.is_none()
                || node
                    .parent
                    .as_ref()
                    .map(|p| p == "client-created-root")
                    .unwrap_or(false)
        })
        .or_else(|| mapping.get("client-created-root"));

    if let Some(root) = root_node {
        traverse_messages(mapping, &root.id, &mut messages, &mut visited, None);
    }

    messages
}

fn traverse_messages(
    mapping: &HashMap<String, MappingNode>,
    node_id: &str,
    messages: &mut Vec<(String, Message, Option<String>)>,
    visited: &mut std::collections::HashSet<String>,
    parent_id: Option<String>,
) {
    if visited.contains(node_id) {
        return;
    }
    visited.insert(node_id.to_string());

    if let Some(node) = mapping.get(node_id) {
        if let Some(msg) = &node.message {
            messages.push((msg.id.clone(), msg.clone(), parent_id));
        }

        for child_id in &node.children {
            traverse_messages(mapping, child_id, messages, visited, Some(node_id.to_string()));
        }
    }
}

fn should_skip_message(msg: &Message) -> bool {
    if let Some(metadata) = &msg.metadata {
        if metadata
            .get("is_visually_hidden_from_conversation")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            return true;
        }
    }

    if msg.content.parts.is_empty()
        || msg
            .content
            .parts
            .iter()
            .all(|part| matches!(part, Value::String(s) if s.trim().is_empty()))
    {
        return true;
    }

    false
}

fn extract_model_slug(msg: &Message, conv: &Conversation) -> Option<String> {
    if msg.author.role == "assistant" {
        if let Some(metadata) = &msg.metadata {
            if let Some(model) = metadata.get("model_slug").and_then(|v| v.as_str()) {
                return Some(model.to_string());
            }
        }
        conv.model_slug.clone()
    } else {
        None
    }
}
