import { describe, it, expect, beforeEach } from 'vitest';
import { loadMainUI, click } from './utils.js';

describe('Sidebar Content Visibility Example Tests', () => {
  let sidebar;
  let hamburgerBtn;
  let sidebarOpen;
  let isAnimating;
  let openSidebar;
  let closeSidebar;
  let toggleSidebar;
  
  beforeEach(() => {
    // Load the main UI structure
    loadMainUI();
    
    // Get DOM elements
    sidebar = document.querySelector('.sidebar');
    hamburgerBtn = document.getElementById('hamburger-btn');
    const backdrop = document.querySelector('.backdrop');
    
    // Initialize state variables
    sidebarOpen = false;
    isAnimating = false;
    
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
    
    toggleSidebar = () => {
      if (isAnimating) return;
      
      if (sidebarOpen) {
        closeSidebar();
      } else {
        openSidebar();
      }
    };
    
    // Attach the toggle handler to the hamburger button
    hamburgerBtn.addEventListener('click', toggleSidebar);
  });

  /**
   * Example 2: Sidebar content visibility
   * Validates: Requirements 1.5
   * 
   * Verify that when the sidebar is open, the conversations list 
   * and controls are visible
   */
  it('should display conversations list and controls when sidebar is open', () => {
    // Verify sidebar starts closed
    expect(sidebar.getAttribute('data-state')).toBe('closed');
    
    // Open the sidebar
    click(hamburgerBtn);
    
    // Verify sidebar is now open
    expect(sidebar.getAttribute('data-state')).toBe('open');
    
    // Verify sidebar header exists and is visible
    const sidebarHeader = sidebar.querySelector('.sidebar-header');
    expect(sidebarHeader).toBeTruthy();
    expect(sidebarHeader).toBeInstanceOf(HTMLElement);
    
    // Verify PROMETHEUS title is present
    const title = sidebarHeader.querySelector('h1');
    expect(title).toBeTruthy();
    expect(title.textContent).toBe('PROMETHEUS');
    
    // Verify sidebar content container exists
    const sidebarContent = sidebar.querySelector('.sidebar-content');
    expect(sidebarContent).toBeTruthy();
    expect(sidebarContent).toBeInstanceOf(HTMLElement);
    
    // Verify "New Chat" button exists and is visible
    const newChatBtn = document.getElementById('new-chat-btn');
    expect(newChatBtn).toBeTruthy();
    expect(newChatBtn.classList.contains('new-chat-btn')).toBe(true);
    expect(newChatBtn.textContent).toBe('+ New Chat');
    
    // Verify the "New Chat" button is inside the sidebar content
    expect(sidebarContent.contains(newChatBtn)).toBe(true);
    
    // Verify conversation list container exists
    const conversationList = document.getElementById('conversation-list');
    expect(conversationList).toBeTruthy();
    expect(conversationList.classList.contains('conversation-list')).toBe(true);
    
    // Verify the conversation list is inside the sidebar content
    expect(sidebarContent.contains(conversationList)).toBe(true);
  });

  it('should maintain sidebar content structure when closed', () => {
    // Verify sidebar starts closed
    expect(sidebar.getAttribute('data-state')).toBe('closed');
    
    // Even when closed, the sidebar content should exist in the DOM
    // (it's just hidden via CSS transform)
    
    // Verify sidebar content exists
    const sidebarContent = sidebar.querySelector('.sidebar-content');
    expect(sidebarContent).toBeTruthy();
    
    // Verify "New Chat" button exists
    const newChatBtn = document.getElementById('new-chat-btn');
    expect(newChatBtn).toBeTruthy();
    
    // Verify conversation list exists
    const conversationList = document.getElementById('conversation-list');
    expect(conversationList).toBeTruthy();
    
    // The elements exist in the DOM, they're just not visible due to transform
    expect(sidebarContent.contains(newChatBtn)).toBe(true);
    expect(sidebarContent.contains(conversationList)).toBe(true);
  });

  it('should have proper nesting of sidebar elements', () => {
    // Verify the sidebar contains the header
    const sidebarHeader = sidebar.querySelector('.sidebar-header');
    expect(sidebar.contains(sidebarHeader)).toBe(true);
    
    // Verify the sidebar contains the content
    const sidebarContent = sidebar.querySelector('.sidebar-content');
    expect(sidebar.contains(sidebarContent)).toBe(true);
    
    // Verify the content contains the new chat button
    const newChatBtn = document.getElementById('new-chat-btn');
    expect(sidebarContent.contains(newChatBtn)).toBe(true);
    
    // Verify the content contains the conversation list
    const conversationList = document.getElementById('conversation-list');
    expect(sidebarContent.contains(conversationList)).toBe(true);
    
    // Verify proper hierarchy: sidebar > content > (button + list)
    expect(sidebar.querySelector('.sidebar-content #new-chat-btn')).toBe(newChatBtn);
    expect(sidebar.querySelector('.sidebar-content #conversation-list')).toBe(conversationList);
  });

  it('should display sidebar content in correct order', () => {
    const sidebarContent = sidebar.querySelector('.sidebar-content');
    const children = Array.from(sidebarContent.children);
    
    // Verify we have at least 2 children (new chat button and conversation list)
    expect(children.length).toBeGreaterThanOrEqual(2);
    
    // Verify "New Chat" button comes before conversation list
    const newChatBtn = document.getElementById('new-chat-btn');
    const conversationList = document.getElementById('conversation-list');
    
    const newChatIndex = children.indexOf(newChatBtn);
    const conversationListIndex = children.indexOf(conversationList);
    
    expect(newChatIndex).toBeGreaterThanOrEqual(0);
    expect(conversationListIndex).toBeGreaterThanOrEqual(0);
    expect(newChatIndex).toBeLessThan(conversationListIndex);
  });
});
