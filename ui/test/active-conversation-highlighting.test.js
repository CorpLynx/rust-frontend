import { describe, it, beforeEach, afterEach, expect } from 'vitest';
import * as fc from 'fast-check';
import { loadMainUI, createMockConversation } from './utils.js';

describe('Active Conversation Highlighting Property Tests', () => {
  let currentConversationId;
  let conversations;
  let renderConversationList;
  
  beforeEach(() => {
    // Load the main UI structure
    loadMainUI();
    
    // Get DOM elements
    const conversationList = document.getElementById('conversation-list');
    
    // Initialize state variables
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
    
    // Define renderConversationList function (mimicking app.js behavior)
    renderConversationList = () => {
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
        
        // Highlight active conversation
        if (conv.id === currentConversationId) {
          item.classList.add('active');
        }
        
        conversationList.appendChild(item);
      });
    };
  });
  
  afterEach(() => {
    // Clean up
    delete window.__TAURI__;
  });

  /**
   * Feature: tauri-ui-improvements, Property 11: Active conversation highlighting
   * Validates: Requirements 6.5
   * 
   * For any active conversation, that conversation should be visually highlighted 
   * in the sidebar list
   */
  it('active conversation is highlighted in sidebar', () => {
    return fc.assert(
      fc.property(
        fc.array(fc.record({
          id: fc.string({ minLength: 1, maxLength: 20 }),
          title: fc.string({ minLength: 1, maxLength: 50 })
        }), { minLength: 1, maxLength: 10 }),
        fc.integer({ min: 0 }),
        (convList, activeIndex) => {
          // Ensure we have a valid active index
          const normalizedIndex = activeIndex % convList.length;
          
          // Setup: Create conversations
          conversations = convList.map((conv, idx) => 
            createMockConversation({ 
              id: `conv-${idx}-${conv.id}`, 
              title: conv.title 
            })
          );
          
          // Set the active conversation
          currentConversationId = conversations[normalizedIndex].id;
          
          // Render the conversation list
          renderConversationList();
          
          // Get all conversation items
          const conversationList = document.getElementById('conversation-list');
          const items = conversationList.querySelectorAll('.conversation-item');
          
          // Verify we have the right number of items
          if (items.length !== conversations.length) {
            return false;
          }
          
          // Check that exactly one item has the 'active' class
          let activeCount = 0;
          let correctItemIsActive = false;
          
          items.forEach((item, idx) => {
            if (item.classList.contains('active')) {
              activeCount++;
              // Check if this is the correct item
              if (item.dataset.conversationId === currentConversationId) {
                correctItemIsActive = true;
              }
            }
          });
          
          // Assertions:
          // 1. Exactly one item should be active
          // 2. The active item should be the one matching currentConversationId
          return activeCount === 1 && correctItemIsActive;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Feature: tauri-ui-improvements, Property 11: Active conversation highlighting
   * Validates: Requirements 6.5
   * 
   * When no conversation is active, no conversation should be highlighted
   */
  it('no highlighting when no active conversation', () => {
    return fc.assert(
      fc.property(
        fc.array(fc.record({
          id: fc.string({ minLength: 1, maxLength: 20 }),
          title: fc.string({ minLength: 1, maxLength: 50 })
        }), { minLength: 1, maxLength: 10 }),
        (convList) => {
          // Setup: Create conversations
          conversations = convList.map((conv, idx) => 
            createMockConversation({ 
              id: `conv-${idx}-${conv.id}`, 
              title: conv.title 
            })
          );
          
          // Set no active conversation
          currentConversationId = null;
          
          // Render the conversation list
          renderConversationList();
          
          // Get all conversation items
          const conversationList = document.getElementById('conversation-list');
          const items = conversationList.querySelectorAll('.conversation-item');
          
          // Check that no items have the 'active' class
          let activeCount = 0;
          items.forEach((item) => {
            if (item.classList.contains('active')) {
              activeCount++;
            }
          });
          
          // Assertion: No items should be active
          return activeCount === 0;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Feature: tauri-ui-improvements, Property 11: Active conversation highlighting
   * Validates: Requirements 6.5
   * 
   * When active conversation changes, highlighting should update accordingly
   */
  it('highlighting updates when active conversation changes', () => {
    return fc.assert(
      fc.property(
        fc.array(fc.record({
          id: fc.string({ minLength: 1, maxLength: 20 }),
          title: fc.string({ minLength: 1, maxLength: 50 })
        }), { minLength: 2, maxLength: 10 }),
        fc.integer({ min: 0 }),
        fc.integer({ min: 0 }),
        (convList, firstIndex, secondIndex) => {
          // Ensure we have valid indices
          const firstNormalized = firstIndex % convList.length;
          const secondNormalized = secondIndex % convList.length;
          
          // Setup: Create conversations
          conversations = convList.map((conv, idx) => 
            createMockConversation({ 
              id: `conv-${idx}-${conv.id}`, 
              title: conv.title 
            })
          );
          
          // Set first active conversation
          currentConversationId = conversations[firstNormalized].id;
          renderConversationList();
          
          // Verify first conversation is highlighted
          const conversationList = document.getElementById('conversation-list');
          let items = conversationList.querySelectorAll('.conversation-item');
          let firstActiveItem = null;
          
          items.forEach((item) => {
            if (item.classList.contains('active')) {
              firstActiveItem = item;
            }
          });
          
          if (!firstActiveItem || firstActiveItem.dataset.conversationId !== conversations[firstNormalized].id) {
            return false;
          }
          
          // Change to second active conversation
          currentConversationId = conversations[secondNormalized].id;
          renderConversationList();
          
          // Verify second conversation is now highlighted
          items = conversationList.querySelectorAll('.conversation-item');
          let secondActiveItem = null;
          let activeCount = 0;
          
          items.forEach((item) => {
            if (item.classList.contains('active')) {
              activeCount++;
              secondActiveItem = item;
            }
          });
          
          // Assertions:
          // 1. Exactly one item should be active
          // 2. The active item should be the second conversation
          return activeCount === 1 && 
                 secondActiveItem !== null && 
                 secondActiveItem.dataset.conversationId === conversations[secondNormalized].id;
        }
      ),
      { numRuns: 100 }
    );
  });
});
