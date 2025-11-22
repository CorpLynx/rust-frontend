import { describe, it, expect, beforeEach } from 'vitest';
import { loadMainUI } from './utils.js';

describe('Hamburger Icon Example Tests', () => {
  beforeEach(() => {
    // Load the main UI structure
    loadMainUI();
  });

  /**
   * Example 1: Hamburger icon presence on load
   * Validates: Requirements 1.1
   * 
   * Verify that the hamburger icon exists in the top-left corner 
   * when the application loads
   */
  it('should display hamburger icon in top-left corner on load', () => {
    // Get the hamburger button element
    const hamburgerBtn = document.getElementById('hamburger-btn');
    
    // Verify the button exists
    expect(hamburgerBtn).toBeTruthy();
    expect(hamburgerBtn).toBeInstanceOf(HTMLElement);
    
    // Verify it has the correct class
    expect(hamburgerBtn.classList.contains('hamburger-btn')).toBe(true);
    
    // Verify it has proper ARIA label for accessibility
    expect(hamburgerBtn.getAttribute('aria-label')).toBe('Toggle conversations menu');
    
    // Verify the hamburger icon span exists inside the button
    const hamburgerIcon = hamburgerBtn.querySelector('.hamburger-icon');
    expect(hamburgerIcon).toBeTruthy();
    expect(hamburgerIcon.tagName).toBe('SPAN');
    
    // Verify the button is in the chat container (top-left positioning context)
    const chatContainer = document.querySelector('.chat-container');
    expect(chatContainer.contains(hamburgerBtn)).toBe(true);
  });
});
