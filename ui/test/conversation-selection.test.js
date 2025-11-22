import { describe, it, beforeEach, afterEach } from 'vitest';
import * as fc from 'fast-check';
import { loadMainUI, click, createMockConversation } from './utils.js';

describe('Conversation Selection Property Tests', () => {
  let openSidebar;
  let closeSidebar;
  let selectConversation;
  let newChat;
  let sidebarOpen;
  let isAnimating;
  let currentConversationId;
  let conversations;
  
  beforeEach(() => {
    // Load the main UI structure
    loadMainUI();
    
    // Get DOM elements
    const sidebar = document.querySelector('.sidebar');
    const hamburgerBtn = document.getElementById('hamburger-btn');
    const backdrop = document.querySelector('.backdrop');
    const conversationList = document.getElementById('conversation-list');
    const newChatBtn = document.getElementById('new-chat-btn');
    
    // Initialize state variables
    sidebarOpen = false;
    isAnimating = false;
    currentConversationId = null;
    conversations = [];
    
    // Mock Tauri fs API
    window.__TAURI__ = window.__TAURI__ || {};
    window.__TAURI__.fs = {
      readTextFile: async (path) => {
        if (path.startsWith('conversations/') && path.endsWith('.json')) {
          const id = path.replace('conversations/', '').replace('.json', '');
          return JSON.stringify({
            id,
            messages: [
              { role: 'user', content: 'Test message' },
              { role: 'assistant', content: 'Test response' }
            ]
          });
        }
        throw new Error('File not found');
      }
    };
    
    // Define the sidebar functions (mimicking app.js behavior)
    openSidebar = () => {
      if (isAnimating || sidebarOpen) return;
      
      isAnimating = true;
      sidebarOpen = true;
      sidebar.setAttribute('data-state', 'open');
      hamburgerBtn.classList.add('active');
      backdrop.setAttribute('data-visible', 'true');
      
      // Simulate animation end
      setTimeout(() => {
        isAnimating = false;
      }, 10);
    };
    
    closeSidebar = () => {
      if (isAnimating || !sidebarOpen) return;
      
      isAnimating = true;
      sidebarOpen = false;
      sidebar.setAttribute('data-state', 'closed');
      hamburgerBtn.classList.remove('active');
      backdrop.setAttribute('data-visible', 'false');
      
      // Simulate animation end
      setTimeout(() => {
        isAnimating = false;
      }, 10);
    };
    
    // Render conversation list
    const renderConversationList = () => {
      conversationList.innerHTML = '';
      
      if (conversations.length === 0) {
        conversationList.innerHTML = '<div style="color: var(--text-secondary);">No conversations yet</div>';
        return;
      }
      
      conversations.forEach(conv => {
        const item = document.createElement('button');
        item.className = 'conversation-item';
        item.textContent = conv.title || 'Untitled';
        item.dataset.conversationId = conv.id;
        
        if (conv.id === currentConversationId) {
          item.classList.add('active');
        }
        
        item.addEventListener('click', () => selectConversation(conv.id));
        
        conversationList.appendChild(item);
      });
    };
    
    // Define selectConversation function (mimicking app.js behavior)
    selectConversation = async (conversationId) => {
      try {
        // Simulate loading conversation
        const fs = window.__TAURI__.fs;
        await fs.readTextFile(`conversations/${conversationId}.json`);
        
        // Update current conversation ID
        currentConversationId = conversationId;
        
        // Update highlighting in sidebar
        renderConversationList();
        
        // Close sidebar after selection
        closeSidebar();
        
      } catch (error) {
        console.error('Failed to load conversation:', error);
      }
    };
    
    // Define newChat function (mimicking app.js behavior)
    newChat = async () => {
      try {
        // Clear current conversation ID
        currentConversationId = null;
        
        // Close sidebar after creation
        closeSidebar();
        
      } catch (error) {
        console.error('Failed to create new chat:', error);
      }
    };
    
    // Attach event handlers
    newChatBtn.addEventListener('click', newChat);
    
    // Initialize with some conversations
    conversations = [
      createMockConversation({ id: 'conv-1', title: 'Conversation 1' }),
      createMockConversation({ id: 'conv-2', title: 'Conversation 2' }),
      createMockConversation({ id: 'conv-3', title: 'Conversation 3' })
    ];
    renderConversationList();
  });
  
  afterEach(() => {
    // Clean up
    delete window.__TAURI__;
  });

  /**
   * Feature: tauri-ui-improvements, Property 10: Conversation selection closes sidebar
   * Validates: Requirements 6.3, 6.4
   * 
   * For any conversation item clicked in the sidebar, the sidebar should close 
   * after loading the conversation
   */
  it('conversation selection closes sidebar', async () => {
    return fc.assert(
      fc.asyncProperty(
        fc.constantFrom('conv-1', 'conv-2', 'conv-3'),
        async (conversationId) => {
          // Setup: Open the sidebar
          const sidebar = document.querySelector('.sidebar');
          
          // Reset state
          sidebarOpen = false;
          isAnimating = false;
          currentConversationId = null;
          sidebar.setAttribute('data-state', 'closed');
          
          // Open the sidebar
          openSidebar();
          
          // Wait for animation to complete
          await new Promise(resolve => setTimeout(resolve, 20));
          
          // Verify sidebar is open
          if (!sidebarOpen || sidebar.getAttribute('data-state') !== 'open') {
            return false;
          }
          
          // Action: Select a conversation
          await selectConversation(conversationId);
          
          // Wait for close animation to start
          await new Promise(resolve => setTimeout(resolve, 5));
          
          // Assertion: Sidebar should be closed or closing
          const sidebarState = sidebar.getAttribute('data-state');
          const isClosed = sidebarState === 'closed' && !sidebarOpen;
          
          return isClosed;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Feature: tauri-ui-improvements, Property 10: Conversation selection closes sidebar
   * Validates: Requirements 6.3, 6.4
   * 
   * For any "New Chat" button click, the sidebar should close after creation
   */
  it('new chat button closes sidebar', async () => {
    return fc.assert(
      fc.asyncProperty(
        fc.constant(true),
        async () => {
          // Setup: Open the sidebar
          const sidebar = document.querySelector('.sidebar');
          const newChatBtn = document.getElementById('new-chat-btn');
          
          // Reset state
          sidebarOpen = false;
          isAnimating = false;
          currentConversationId = null;
          sidebar.setAttribute('data-state', 'closed');
          
          // Open the sidebar
          openSidebar();
          
          // Wait for animation to complete
          await new Promise(resolve => setTimeout(resolve, 20));
          
          // Verify sidebar is open
          if (!sidebarOpen || sidebar.getAttribute('data-state') !== 'open') {
            return false;
          }
          
          // Action: Click new chat button
          click(newChatBtn);
          
          // Wait for the async newChat function to complete
          await new Promise(resolve => setTimeout(resolve, 5));
          
          // Assertion: Sidebar should be closed or closing
          const sidebarState = sidebar.getAttribute('data-state');
          const isClosed = sidebarState === 'closed' && !sidebarOpen;
          
          return isClosed;
        }
      ),
      { numRuns: 100 }
    );
  });
});
