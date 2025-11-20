import fs from 'fs';
import path from 'path';
import { v4 as uuidv4 } from 'uuid';

const CONVERSATIONS_DIR = 'conversations';
const METADATA_FILE = path.join(CONVERSATIONS_DIR, 'metadata.json');

export class Conversation {
  constructor(name, model = null) {
    this.id = uuidv4();
    this.name = name;
    this.messages = [];
    this.createdAt = new Date().toISOString();
    this.updatedAt = new Date().toISOString();
    this.model = model;
  }

  static withTimestampName(model = null) {
    const timestamp = new Date().toLocaleString();
    return new Conversation(`Chat ${timestamp}`, model);
  }

  updateTimestamp() {
    this.updatedAt = new Date().toISOString();
  }

  addMessage(message) {
    this.messages.push(message);
    this.updateTimestamp();
  }

  removeMessage(index) {
    if (index < this.messages.length) {
      this.messages.splice(index, 1);
      this.updateTimestamp();
    }
  }

  updateMessage(index, content) {
    if (index < this.messages.length) {
      this.messages[index].content = content;
      this.updateTimestamp();
    }
  }
}

export class ConversationManager {
  constructor() {
    this.conversationsDir = CONVERSATIONS_DIR;
    this.metadataPath = METADATA_FILE;
    this.ensureDirectories();
  }

  ensureDirectories() {
    if (!fs.existsSync(this.conversationsDir)) {
      fs.mkdirSync(this.conversationsDir, { recursive: true });
    }
    if (!fs.existsSync(this.metadataPath)) {
      fs.writeFileSync(this.metadataPath, JSON.stringify({ conversations: [] }, null, 2));
    }
  }

  loadMetadata() {
    try {
      const content = fs.readFileSync(this.metadataPath, 'utf8');
      const metadata = JSON.parse(content);
      metadata.conversations.sort((a, b) => 
        new Date(b.updatedAt) - new Date(a.updatedAt)
      );
      return metadata;
    } catch (error) {
      return { conversations: [] };
    }
  }

  saveMetadata(metadata) {
    try {
      fs.writeFileSync(this.metadataPath, JSON.stringify(metadata, null, 2));
      return true;
    } catch (error) {
      console.error('Failed to save metadata:', error);
      return false;
    }
  }

  loadConversation(id) {
    try {
      const filePath = path.join(this.conversationsDir, `${id}.json`);
      const content = fs.readFileSync(filePath, 'utf8');
      return JSON.parse(content);
    } catch (error) {
      throw new Error(`Failed to load conversation: ${error.message}`);
    }
  }

  saveConversation(conversation) {
    try {
      const filePath = path.join(this.conversationsDir, `${conversation.id}.json`);
      fs.writeFileSync(filePath, JSON.stringify(conversation, null, 2));

      // Update metadata
      const metadata = this.loadMetadata();
      const conversationMeta = this.createMetadata(conversation);
      
      const existingIndex = metadata.conversations.findIndex(c => c.id === conversation.id);
      if (existingIndex >= 0) {
        metadata.conversations[existingIndex] = conversationMeta;
      } else {
        metadata.conversations.push(conversationMeta);
      }
      
      this.saveMetadata(metadata);
      return true;
    } catch (error) {
      console.error('Failed to save conversation:', error);
      return false;
    }
  }

  deleteConversation(id) {
    try {
      const filePath = path.join(this.conversationsDir, `${id}.json`);
      if (fs.existsSync(filePath)) {
        fs.unlinkSync(filePath);
      }

      const metadata = this.loadMetadata();
      metadata.conversations = metadata.conversations.filter(c => c.id !== id);
      this.saveMetadata(metadata);
      return true;
    } catch (error) {
      console.error('Failed to delete conversation:', error);
      return false;
    }
  }

  listConversations() {
    const metadata = this.loadMetadata();
    return metadata.conversations;
  }

  createMetadata(conversation) {
    let preview = 'Empty conversation';
    if (conversation.messages.length > 0) {
      const firstUserMsg = conversation.messages.find(m => m.role === 'user');
      const content = firstUserMsg ? firstUserMsg.content : conversation.messages[0].content;
      preview = content.length > 50 ? content.substring(0, 50) + '...' : content;
    }

    return {
      id: conversation.id,
      name: conversation.name,
      preview,
      updatedAt: conversation.updatedAt,
      messageCount: conversation.messages.length
    };
  }
}
