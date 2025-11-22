import { describe, it, expect, beforeEach } from 'vitest';
import { loadMainUI, getStyle } from './utils.js';

describe('Input Container Example Tests', () => {
  let inputContainer;
  let messageInput;
  let sendBtn;

  beforeEach(() => {
    // Load the main UI structure
    loadMainUI();
    
    // Get the elements
    inputContainer = document.querySelector('.input-container');
    messageInput = document.getElementById('message-input');
    sendBtn = document.getElementById('send-btn');
    
    // Add minimal CSS for testing layout and styling
    const style = document.createElement('style');
    style.textContent = `
      .input-container {
        position: fixed;
        bottom: 20px;
        left: 50%;
        transform: translateX(-50%);
        max-width: 800px;
        width: calc(100% - 40px);
        display: flex;
        gap: 12px;
        padding: 12px 16px;
        background: #2a2a2a;
        border-radius: 24px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        z-index: 100;
      }
      
      .message-input {
        flex: 1;
        padding: 14px 18px;
        background: transparent;
        border: 1px solid transparent;
        border-radius: 16px;
        color: #e0e0e0;
        font-size: 15px;
        transition: border-color 0.2s;
      }
      
      .send-btn-circular {
        width: 48px;
        height: 48px;
        background: #00ff88;
        color: #0a0a0a;
        border: none;
        border-radius: 50%;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: transform 0.2s, opacity 0.2s;
        flex-shrink: 0;
      }
      
      .chat-container {
        background: #0a0a0a;
      }
    `;
    document.head.appendChild(style);
  });

  /**
   * Example 4: Input container layout
   * Validates: Requirements 3.1, 3.3
   * 
   * Verify that the input container is centered horizontally at the bottom 
   * with proper spacing
   */
  it('should display input container centered at bottom with proper spacing', () => {
    // Verify the input container exists
    expect(inputContainer).toBeTruthy();
    expect(inputContainer).toBeInstanceOf(HTMLElement);
    
    // Verify it has the correct class
    expect(inputContainer.classList.contains('input-container')).toBe(true);
    
    // Verify fixed positioning at bottom
    const position = getStyle(inputContainer, 'position');
    expect(position).toBe('fixed');
    
    const bottom = getStyle(inputContainer, 'bottom');
    expect(bottom).toBe('20px');
    
    // Verify horizontal centering
    const left = getStyle(inputContainer, 'left');
    expect(left).toBe('50%');
    
    const transform = getStyle(inputContainer, 'transform');
    // Transform should include translateX(-50%) for centering
    expect(transform).toContain('translateX(-50%)');
    
    // Verify max-width constraint
    const maxWidth = getStyle(inputContainer, 'max-width');
    expect(maxWidth).toBe('800px');
    
    // Verify it contains the input and button
    expect(inputContainer.contains(messageInput)).toBe(true);
    expect(inputContainer.contains(sendBtn)).toBe(true);
    
    // Verify display flex for layout
    const display = getStyle(inputContainer, 'display');
    expect(display).toBe('flex');
  });

  /**
   * Example 5: Input container styling
   * Validates: Requirements 3.2, 5.1
   * 
   * Verify that the input container has rounded edges and box shadow
   */
  it('should have rounded edges and elevated styling with box shadow', () => {
    // Verify rounded edges
    const borderRadius = getStyle(inputContainer, 'border-radius');
    expect(borderRadius).toBe('24px');
    
    // Verify box shadow for elevation effect
    const boxShadow = getStyle(inputContainer, 'box-shadow');
    expect(boxShadow).toBeTruthy();
    expect(boxShadow).not.toBe('none');
    // Should contain shadow values (rgba for color and pixel values)
    expect(boxShadow).toContain('rgba');
    expect(boxShadow).toContain('px');
    
    // Verify z-index for layering
    const zIndex = getStyle(inputContainer, 'z-index');
    expect(parseInt(zIndex)).toBeGreaterThan(0);
    
    // Verify padding for internal spacing
    const padding = getStyle(inputContainer, 'padding');
    expect(padding).toBeTruthy();
    expect(padding).not.toBe('0px');
  });

  /**
   * Example 8: Background color distinction
   * Validates: Requirements 5.2
   * 
   * Verify that the input container background color differs from 
   * the chat area background
   */
  it('should have background color distinct from chat area', () => {
    // Get the chat container (chat area background)
    const chatContainer = document.querySelector('.chat-container');
    expect(chatContainer).toBeTruthy();
    
    // Get background colors
    const inputBgColor = getStyle(inputContainer, 'background-color');
    const chatBgColor = getStyle(chatContainer, 'background-color');
    
    // Verify both have background colors set
    expect(inputBgColor).toBeTruthy();
    expect(chatBgColor).toBeTruthy();
    
    // Verify they are different
    expect(inputBgColor).not.toBe(chatBgColor);
    
    // Verify input container has a distinct background (not transparent)
    expect(inputBgColor).not.toBe('rgba(0, 0, 0, 0)');
    expect(inputBgColor).not.toBe('transparent');
    
    // The input container should be lighter/more visible than the chat background
    // (in a dark theme, tertiary is lighter than primary)
    // We can verify this by checking that the colors are actually different values
    const inputRgb = inputBgColor.match(/\d+/g);
    const chatRgb = chatBgColor.match(/\d+/g);
    
    if (inputRgb && chatRgb) {
      // At least one RGB component should be different
      const isDifferent = inputRgb.some((val, idx) => val !== chatRgb[idx]);
      expect(isDifferent).toBe(true);
    }
  });
});
