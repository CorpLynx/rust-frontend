use crate::app::ChatMessage;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub name: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: String,
    pub updated_at: String,
    pub model: Option<String>,
}

impl Conversation {
    pub fn new(name: String, model: Option<String>) -> Self {
        let now = chrono::Local::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            messages: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
            model,
        }
    }

    pub fn with_timestamp_name(model: Option<String>) -> Self {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self::new(format!("Chat {}", timestamp), model)
    }

    pub fn update_timestamp(&mut self) {
        self.updated_at = chrono::Local::now().to_rfc3339();
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        self.update_timestamp();
    }

    pub fn remove_message(&mut self, index: usize) {
        if index < self.messages.len() {
            self.messages.remove(index);
            self.update_timestamp();
        }
    }

    pub fn update_message(&mut self, index: usize, content: String) {
        if index < self.messages.len() {
            self.messages[index].content = content;
            self.update_timestamp();
        }
    }

    pub fn clear_messages_after(&mut self, index: usize) {
        if index < self.messages.len() {
            self.messages.truncate(index + 1);
            self.update_timestamp();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub id: String,
    pub name: String,
    pub preview: String,
    pub updated_at: String,
    pub message_count: usize,
}

impl ConversationMetadata {
    pub fn from_conversation(conv: &Conversation) -> Self {
        let preview = if conv.messages.is_empty() {
            "Empty conversation".to_string()
        } else {
            let first_user_msg = conv
                .messages
                .iter()
                .find(|m| m.role == "user")
                .map(|m| &m.content)
                .unwrap_or(&conv.messages[0].content);
            
            if first_user_msg.len() > 50 {
                format!("{}...", &first_user_msg[..50])
            } else {
                first_user_msg.clone()
            }
        };

        Self {
            id: conv.id.clone(),
            name: conv.name.clone(),
            preview,
            updated_at: conv.updated_at.clone(),
            message_count: conv.messages.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataFile {
    pub conversations: Vec<ConversationMetadata>,
}

impl MetadataFile {
    pub fn new() -> Self {
        Self {
            conversations: Vec::new(),
        }
    }

    pub fn add_or_update(&mut self, metadata: ConversationMetadata) {
        if let Some(pos) = self.conversations.iter().position(|m| m.id == metadata.id) {
            self.conversations[pos] = metadata;
        } else {
            self.conversations.push(metadata);
        }
        self.sort_by_date();
    }

    pub fn remove(&mut self, id: &str) {
        self.conversations.retain(|m| m.id != id);
    }

    pub fn sort_by_date(&mut self) {
        self.conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    }
}

#[derive(Clone)]
pub struct ConversationManager {
    conversations_dir: PathBuf,
    metadata_path: PathBuf,
}

impl ConversationManager {
    pub fn new() -> Self {
        let conversations_dir = PathBuf::from("conversations");
        let metadata_path = conversations_dir.join("metadata.json");
        Self {
            conversations_dir,
            metadata_path,
        }
    }

    pub fn load_metadata(&self) -> Result<MetadataFile> {
        if !self.metadata_path.exists() {
            return Ok(MetadataFile::new());
        }

        let content = fs::read_to_string(&self.metadata_path)
            .context("Failed to read metadata file")?;
        
        let mut metadata: MetadataFile = serde_json::from_str(&content)
            .context("Failed to parse metadata file")?;
        
        metadata.sort_by_date();
        Ok(metadata)
    }

    pub fn save_metadata(&self, metadata: &MetadataFile) -> Result<()> {
        let content = serde_json::to_string_pretty(metadata)
            .context("Failed to serialize metadata")?;
        
        fs::write(&self.metadata_path, content)
            .context("Failed to write metadata file")?;
        
        Ok(())
    }

    pub fn load_conversation(&self, id: &str) -> Result<Conversation> {
        let path = self.conversations_dir.join(format!("{}.json", id));
        
        if !path.exists() {
            anyhow::bail!("Conversation file not found: {}", id);
        }

        let content = fs::read_to_string(&path)
            .context("Failed to read conversation file")?;
        
        let conversation: Conversation = serde_json::from_str(&content)
            .context("Failed to parse conversation file")?;
        
        Ok(conversation)
    }

    pub fn save_conversation(&self, conversation: &Conversation) -> Result<()> {
        let path = self.conversations_dir.join(format!("{}.json", conversation.id));
        
        let content = serde_json::to_string_pretty(conversation)
            .context("Failed to serialize conversation")?;
        
        fs::write(&path, content)
            .context("Failed to write conversation file")?;

        // Update metadata
        let mut metadata = self.load_metadata()?;
        metadata.add_or_update(ConversationMetadata::from_conversation(conversation));
        self.save_metadata(&metadata)?;
        
        Ok(())
    }

    pub fn delete_conversation(&self, id: &str) -> Result<()> {
        let path = self.conversations_dir.join(format!("{}.json", id));
        
        if path.exists() {
            fs::remove_file(&path)
                .context("Failed to delete conversation file")?;
        }

        // Update metadata
        let mut metadata = self.load_metadata()?;
        metadata.remove(id);
        self.save_metadata(&metadata)?;
        
        Ok(())
    }

    pub fn list_conversations(&self) -> Result<Vec<ConversationMetadata>> {
        let metadata = self.load_metadata()?;
        Ok(metadata.conversations)
    }
}
