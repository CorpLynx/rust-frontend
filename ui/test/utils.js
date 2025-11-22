import { screen, waitFor } from '@testing-library/dom';

/**
 * Load HTML fixture into the DOM
 * @param {string} html - HTML string to load
 */
export function loadHTML(html) {
  document.body.innerHTML = html;
}

/**
 * Load the main UI HTML structure
 */
export function loadMainUI() {
  const html = `
    <div class="app">
      <div class="backdrop" data-visible="false"></div>
      
      <div class="sidebar" data-state="closed">
        <div class="sidebar-header">
          <h1>PROMETHEUS</h1>
        </div>
        <div class="sidebar-content">
          <button id="new-chat-btn" class="new-chat-btn">+ New Chat</button>
          <div id="conversation-list" class="conversation-list"></div>
        </div>
      </div>
      
      <div class="chat-container">
        <button id="hamburger-btn" class="hamburger-btn" aria-label="Toggle conversations menu">
          <span class="hamburger-icon"></span>
        </button>
        
        <div id="messages" class="messages"></div>
        
        <div class="input-container">
          <input 
            type="text" 
            id="message-input" 
            class="message-input"
            placeholder="Type a message..." 
            autocomplete="off"
          />
          <button id="send-btn" class="send-btn-circular" aria-label="Send message">
            <svg class="send-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M22 2L11 13" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              <path d="M22 2L15 22L11 13L2 9L22 2Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
  `;
  loadHTML(html);
}

/**
 * Simulate a click event on an element
 * @param {HTMLElement} element - Element to click
 */
export function click(element) {
  const event = new MouseEvent('click', {
    bubbles: true,
    cancelable: true
  });
  element.dispatchEvent(event);
}

/**
 * Simulate a hover event on an element
 * @param {HTMLElement} element - Element to hover
 */
export function hover(element) {
  const event = new MouseEvent('mouseenter', {
    bubbles: true,
    cancelable: true
  });
  element.dispatchEvent(event);
}

/**
 * Simulate typing in an input element
 * @param {HTMLInputElement} input - Input element
 * @param {string} text - Text to type
 */
export function typeText(input, text) {
  input.value = text;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  input.dispatchEvent(new Event('change', { bubbles: true }));
}

/**
 * Wait for an animation to complete
 * @param {number} duration - Animation duration in ms
 * @returns {Promise<void>}
 */
export function waitForAnimation(duration = 300) {
  return new Promise(resolve => setTimeout(resolve, duration));
}

/**
 * Get computed style property value
 * @param {HTMLElement} element - Element to check
 * @param {string} property - CSS property name
 * @returns {string}
 */
export function getStyle(element, property) {
  return window.getComputedStyle(element).getPropertyValue(property);
}

/**
 * Check if element is visible
 * @param {HTMLElement} element - Element to check
 * @returns {boolean}
 */
export function isVisible(element) {
  const style = window.getComputedStyle(element);
  return style.display !== 'none' && 
         style.visibility !== 'hidden' && 
         style.opacity !== '0';
}

/**
 * Get element by test id
 * @param {string} testId - Test ID
 * @returns {HTMLElement}
 */
export function getByTestId(testId) {
  return document.querySelector(`[data-testid="${testId}"]`);
}

/**
 * Simulate window resize
 * @param {number} width - Window width
 * @param {number} height - Window height
 */
export function resizeWindow(width, height) {
  Object.defineProperty(window, 'innerWidth', {
    writable: true,
    configurable: true,
    value: width
  });
  Object.defineProperty(window, 'innerHeight', {
    writable: true,
    configurable: true,
    value: height
  });
  window.dispatchEvent(new Event('resize'));
}

/**
 * Mock Tauri invoke command
 * @param {string} command - Command name
 * @param {Function} handler - Handler function
 */
export function mockTauriCommand(command, handler) {
  const originalInvoke = window.__TAURI__.core.invoke;
  window.__TAURI__.core.invoke = async (cmd, args) => {
    if (cmd === command) {
      return handler(args);
    }
    return originalInvoke(cmd, args);
  };
}

/**
 * Create a mock conversation object
 * @param {Object} overrides - Property overrides
 * @returns {Object}
 */
export function createMockConversation(overrides = {}) {
  return {
    id: `conv-${Math.random().toString(36).substr(2, 9)}`,
    title: 'Test Conversation',
    lastModified: new Date().toISOString(),
    messageCount: 0,
    ...overrides
  };
}

/**
 * Create a mock message object
 * @param {string} role - Message role ('user' or 'assistant')
 * @param {string} content - Message content
 * @returns {Object}
 */
export function createMockMessage(role = 'user', content = 'Test message') {
  return {
    role,
    content,
    timestamp: new Date().toISOString()
  };
}

// Re-export testing-library utilities
export { screen, waitFor };
