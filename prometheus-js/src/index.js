import blessed from 'blessed';
import { Config } from './config.js';
import { ConversationManager, Conversation } from './conversation.js';
import { OllamaAPI } from './api.js';
import { parseMarkdown, renderSegmentToTerminal } from './markdown.js';

class PrometheusApp {
  constructor() {
    this.config = new Config();
    this.conversationManager = new ConversationManager();
    this.api = new OllamaAPI(
      this.config.get('backend.ollamaUrl'),
      this.config.get('backend.timeoutSeconds') * 1000
    );
    
    this.activeConversation = null;
    this.chatHistory = [];
    this.models = [];
    this.selectedModel = null;
    this.isLoading = false;
    this.streamingResponse = '';
    
    this.setupUI();
    this.setupEventHandlers();
    this.initialize();
  }

  setupUI() {
    const theme = this.config.getTheme();
    
    // Create screen
    this.screen = blessed.screen({
      smartCSR: true,
      title: this.config.get('app.windowTitle'),
      fullUnicode: true
    });

    // Header
    this.header = blessed.box({
      top: 0,
      left: 0,
      width: '100%',
      height: 3,
      content: '{center}{bold}PROMETHEUS{/bold}{/center}',
      tags: true,
      style: {
        fg: theme.primary,
        bg: theme.background,
        border: {
          fg: theme.secondary
        }
      },
      border: {
        type: 'line'
      }
    });

    // Chat display area
    this.chatBox = blessed.box({
      top: 3,
      left: 0,
      width: '100%',
      height: '100%-8',
      scrollable: true,
      alwaysScroll: true,
      scrollbar: {
        ch: '█',
        style: {
          fg: theme.secondary
        }
      },
      tags: true,
      style: {
        fg: theme.text,
        bg: theme.background
      },
      keys: true,
      vi: true,
      mouse: true
    });

    // Input box
    this.inputBox = blessed.textarea({
      bottom: 3,
      left: 0,
      width: '100%-15',
      height: 3,
      inputOnFocus: true,
      style: {
        fg: theme.primary,
        bg: theme.background,
        border: {
          fg: theme.secondary
        }
      },
      border: {
        type: 'line'
      }
    });

    // Send button
    this.sendButton = blessed.button({
      bottom: 3,
      right: 0,
      width: 15,
      height: 3,
      content: '{center}Send{/center}',
      tags: true,
      style: {
        fg: theme.background,
        bg: theme.primary,
        border: {
          fg: theme.secondary
        },
        focus: {
          bg: theme.secondary
        }
      },
      border: {
        type: 'line'
      }
    });

    // Status bar
    this.statusBar = blessed.box({
      bottom: 0,
      left: 0,
      width: '100%',
      height: 3,
      content: '',
      tags: true,
      style: {
        fg: theme.secondary,
        bg: theme.background,
        border: {
          fg: theme.secondary
        }
      },
      border: {
        type: 'line'
      }
    });

    // Add all elements to screen
    this.screen.append(this.header);
    this.screen.append(this.chatBox);
    this.screen.append(this.inputBox);
    this.screen.append(this.sendButton);
    this.screen.append(this.statusBar);

    // Focus input by default
    this.inputBox.focus();
    this.screen.render();
  }

  setupEventHandlers() {
    // Quit on Escape, q, or Ctrl-C
    this.screen.key(['escape', 'q', 'C-c'], () => {
      return process.exit(0);
    });

    // Send message on Enter
    this.inputBox.key('enter', () => {
      if (!this.isLoading) {
        this.sendMessage();
      }
    });

    // Send button click
    this.sendButton.on('press', () => {
      if (!this.isLoading) {
        this.sendMessage();
      }
    });

    // Focus management
    this.screen.key('tab', () => {
      if (this.inputBox.focused) {
        this.sendButton.focus();
      } else {
        this.inputBox.focus();
      }
      this.screen.render();
    });
  }

  async initialize() {
    this.updateStatus('Initializing...');
    
    try {
      // Fetch available models
      this.models = await this.api.fetchModels();
      if (this.models.length > 0) {
        this.selectedModel = this.models[0];
      }
      
      // Create new conversation
      this.activeConversation = Conversation.withTimestampName(this.selectedModel);
      
      this.updateStatus(`Ready | Model: ${this.selectedModel || 'none'} | Press Tab to switch focus, Esc/q to quit`);
      this.showWelcomeMessage();
    } catch (error) {
      this.updateStatus(`Error: ${error.message}`);
    }
    
    this.screen.render();
  }

  showWelcomeMessage() {
    const theme = this.config.getTheme();
    const asciiArt = `
   ___  ___  ___  __  __ ___ _____ _  _ ___ _   _ ___ 
  | _ \\| _ \\/ _ \\|  \\/  | __|_   _| || | __| | | / __|
  |  _/|   / (_) | |\\/| | _|  | | | __ | _|| |_| \\__ \\
  |_|  |_|_\\\\___/|_|  |_|___| |_| |_||_|___|\\___/|___/
    `;
    
    this.chatBox.setContent(`{center}{${theme.primary}-fg}${asciiArt}{/}{/center}\n\n{center}Start chatting by typing a message below{/center}`);
    this.screen.render();
  }

  updateStatus(message) {
    this.statusBar.setContent(`{center}${message}{/center}`);
    this.screen.render();
  }

  async sendMessage() {
    const prompt = this.inputBox.getValue().trim();
    
    if (!prompt) {
      return;
    }

    if (!this.selectedModel) {
      this.updateStatus('Error: No model selected');
      return;
    }

    // Add user message
    const userMessage = {
      role: 'user',
      content: prompt,
      timestamp: new Date().toLocaleTimeString()
    };
    
    this.chatHistory.push(userMessage);
    this.activeConversation.addMessage(userMessage);
    
    // Clear input
    this.inputBox.clearValue();
    
    // Update display
    this.renderChatHistory();
    
    // Set loading state
    this.isLoading = true;
    this.streamingResponse = '';
    this.updateStatus('Generating response...');
    
    try {
      // Stream response
      for await (const chunk of this.api.generateStream(this.selectedModel, prompt)) {
        this.streamingResponse += chunk;
        this.renderChatHistory(true);
      }
      
      // Add AI message
      const aiMessage = {
        role: 'assistant',
        content: this.streamingResponse,
        timestamp: new Date().toLocaleTimeString()
      };
      
      this.chatHistory.push(aiMessage);
      this.activeConversation.addMessage(aiMessage);
      
      // Save conversation
      this.conversationManager.saveConversation(this.activeConversation);
      
      this.streamingResponse = '';
      this.renderChatHistory();
      
    } catch (error) {
      this.updateStatus(`Error: ${error.message}`);
    } finally {
      this.isLoading = false;
      this.updateStatus(`Ready | Model: ${this.selectedModel} | Press Tab to switch focus, Esc/q to quit`);
    }
  }

  renderChatHistory(showStreaming = false) {
    const theme = this.config.getTheme();
    let content = '';
    
    for (const message of this.chatHistory) {
      const isUser = message.role === 'user';
      const roleLabel = isUser ? 'You' : 'AI';
      const roleColor = isUser ? theme.secondary : theme.primary;
      
      content += `\n{${roleColor}-fg}{bold}${roleLabel}{/bold}{/} {gray-fg}[${message.timestamp}]{/}\n`;
      
      // Parse and render markdown
      const segments = parseMarkdown(message.content);
      for (const segment of segments) {
        content += renderSegmentToTerminal(segment, theme) + '\n';
      }
      
      content += '\n';
    }
    
    // Show streaming response
    if (showStreaming && this.streamingResponse) {
      content += `\n{${theme.primary}-fg}{bold}AI{/bold}{/} {gray-fg}[streaming...]{/}\n`;
      
      const segments = parseMarkdown(this.streamingResponse);
      for (const segment of segments) {
        content += renderSegmentToTerminal(segment, theme) + '\n';
      }
    }
    
    this.chatBox.setContent(content);
    this.chatBox.setScrollPerc(100);
    this.screen.render();
  }
}

// Start the application
const app = new PrometheusApp();
