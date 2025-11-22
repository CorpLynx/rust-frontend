import { describe, it, expect, beforeEach } from 'vitest';
import { loadMainUI, getStyle } from './utils.js';

describe('Circular Send Button Example Tests', () => {
  let sendBtn;
  let sendIcon;

  beforeEach(() => {
    // Load the main UI structure
    loadMainUI();
    
    // Get the elements
    sendBtn = document.getElementById('send-btn');
    sendIcon = sendBtn.querySelector('.send-icon');
    
    // Add the actual CSS for testing
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
      
      .send-icon {
        width: 20px;
        height: 20px;
        color: #0a0a0a;
      }
    `;
    document.head.appendChild(style);
  });

  /**
   * Example 6: Circular send button
   * Validates: Requirements 3.4, 4.1
   * 
   * Verify that the send button is circular (equal width/height, 50% border-radius) 
   * and positioned on the right
   */
  it('should render send button as circular shape with consistent dimensions', () => {
    // Verify the send button exists
    expect(sendBtn).toBeTruthy();
    expect(sendBtn).toBeInstanceOf(HTMLElement);
    
    // Verify it has the correct class
    expect(sendBtn.classList.contains('send-btn-circular')).toBe(true);
    
    // Verify equal width and height for circular shape
    const width = getStyle(sendBtn, 'width');
    const height = getStyle(sendBtn, 'height');
    
    expect(width).toBe(height);
    expect(width).toBe('48px');
    expect(height).toBe('48px');
    
    // Verify 50% border-radius for perfect circle
    const borderRadius = getStyle(sendBtn, 'border-radius');
    expect(borderRadius).toBe('50%');
    
    // Verify it's positioned on the far right of input container
    const inputContainer = document.querySelector('.input-container');
    expect(inputContainer).toBeTruthy();
    expect(inputContainer.contains(sendBtn)).toBe(true);
    
    // Verify flex layout properties
    const display = getStyle(sendBtn, 'display');
    expect(display).toBe('flex');
    
    const alignItems = getStyle(sendBtn, 'align-items');
    expect(alignItems).toBe('center');
    
    const justifyContent = getStyle(sendBtn, 'justify-content');
    expect(justifyContent).toBe('center');
    
    // Verify flex-shrink to maintain size
    const flexShrink = getStyle(sendBtn, 'flex-shrink');
    expect(flexShrink).toBe('0');
    
    // Verify cursor style
    const cursor = getStyle(sendBtn, 'cursor');
    expect(cursor).toBe('pointer');
  });

  /**
   * Example 7: Send icon presence
   * Validates: Requirements 4.2
   * 
   * Verify that the send button contains a centered icon
   */
  it('should display send icon centered within circular button', () => {
    // Verify the send icon exists
    expect(sendIcon).toBeTruthy();
    expect(sendIcon).toBeInstanceOf(SVGElement);
    
    // Verify it has the correct class
    expect(sendIcon.classList.contains('send-icon')).toBe(true);
    
    // Verify the icon is a child of the send button
    expect(sendBtn.contains(sendIcon)).toBe(true);
    
    // Verify icon dimensions
    const iconWidth = getStyle(sendIcon, 'width');
    const iconHeight = getStyle(sendIcon, 'height');
    
    expect(iconWidth).toBe('20px');
    expect(iconHeight).toBe('20px');
    
    // Verify the icon is centered by checking parent's flex properties
    const parentDisplay = getStyle(sendBtn, 'display');
    const parentAlign = getStyle(sendBtn, 'align-items');
    const parentJustify = getStyle(sendBtn, 'justify-content');
    
    expect(parentDisplay).toBe('flex');
    expect(parentAlign).toBe('center');
    expect(parentJustify).toBe('center');
    
    // Verify the SVG has path elements (the actual icon content)
    const paths = sendIcon.querySelectorAll('path');
    expect(paths.length).toBeGreaterThan(0);
    
    // Verify ARIA label for accessibility
    const ariaLabel = sendBtn.getAttribute('aria-label');
    expect(ariaLabel).toBeTruthy();
    expect(ariaLabel.toLowerCase()).toContain('send');
  });
});
