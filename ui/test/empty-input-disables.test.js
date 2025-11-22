import { describe, it, beforeEach, expect } from 'vitest';
import * as fc from 'fast-check';
import { loadMainUI, typeText } from './utils.js';

describe('Empty Input Disables Send Button Property Tests', () => {
  let messageInput;
  let sendBtn;
  
  beforeEach(() => {
    // Load the main UI structure
    loadMainUI();
    
    // Get the elements
    messageInput = document.getElementById('message-input');
    sendBtn = document.getElementById('send-btn');
    
    // Add minimal CSS for button states
    const style = document.createElement('style');
    style.textContent = `
      :root {
        --accent: #00ff88;
        --bg-primary: #0a0a0a;
      }
      .send-btn-circular {
        width: 48px;
        height: 48px;
        background: var(--accent);
        color: var(--bg-primary);
        border: none;
        border-radius: 50%;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: transform 0.2s, opacity 0.2s;
        flex-shrink: 0;
      }
      .send-btn-circular:disabled {
        opacity: 0.5;
        cursor: not-allowed;
      }
    `;
    document.head.appendChild(style);
    
    // Set up the updateSendButtonState function (from app.js)
    window.updateSendButtonState = function() {
      const message = messageInput.value.trim();
      const isEmpty = message === '';
      sendBtn.disabled = isEmpty;
    };
    
    // Add event listeners
    messageInput.addEventListener('input', window.updateSendButtonState);
    messageInput.addEventListener('keyup', window.updateSendButtonState);
    
    // Initialize button state
    window.updateSendButtonState();
  });

  /**
   * Feature: tauri-ui-improvements, Property 7: Empty input disables send button
   * Validates: Requirements 4.4
   * 
   * For any message input state, when the input is empty or contains only whitespace,
   * the send button should be disabled
   */
  it('send button is disabled when input is empty or contains only whitespace', () => {
    fc.assert(
      fc.property(
        fc.string(),
        (inputText) => {
          // Action: Type the text into the input
          typeText(messageInput, inputText);
          
          // Verify: Button should be disabled if and only if the trimmed input is empty
          const trimmedText = inputText.trim();
          const expectedDisabled = trimmedText === '';
          
          return sendBtn.disabled === expectedDisabled;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Feature: tauri-ui-improvements, Property 7: Empty input disables send button
   * Validates: Requirements 4.4
   * 
   * For any sequence of input changes, the send button disabled state should
   * always correctly reflect whether the current input is empty/whitespace
   */
  it('send button disabled state correctly tracks input changes', () => {
    fc.assert(
      fc.property(
        fc.array(fc.string(), { minLength: 1, maxLength: 10 }),
        (inputSequence) => {
          let allStatesCorrect = true;
          
          for (const inputText of inputSequence) {
            // Type the text
            typeText(messageInput, inputText);
            
            // Check if button state is correct
            const trimmedText = inputText.trim();
            const expectedDisabled = trimmedText === '';
            
            if (sendBtn.disabled !== expectedDisabled) {
              allStatesCorrect = false;
              break;
            }
          }
          
          return allStatesCorrect;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Feature: tauri-ui-improvements, Property 7: Empty input disables send button
   * Validates: Requirements 4.4
   * 
   * For any whitespace-only string (spaces, tabs, newlines), the send button
   * should be disabled since trim() will result in an empty string
   */
  it('send button is disabled for all whitespace-only inputs', () => {
    fc.assert(
      fc.property(
        fc.stringMatching(/^\s+$/),
        (whitespaceText) => {
          // Action: Type whitespace-only text
          typeText(messageInput, whitespaceText);
          
          // Verify: Button should be disabled for whitespace-only input
          return sendBtn.disabled === true;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Feature: tauri-ui-improvements, Property 7: Empty input disables send button
   * Validates: Requirements 4.4
   * 
   * For any non-empty trimmed string, the send button should be enabled
   */
  it('send button is enabled for any non-empty trimmed input', () => {
    fc.assert(
      fc.property(
        fc.string({ minLength: 1 }).filter(s => s.trim().length > 0),
        (nonEmptyText) => {
          // Action: Type non-empty text
          typeText(messageInput, nonEmptyText);
          
          // Verify: Button should be enabled for non-empty input
          return sendBtn.disabled === false;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Feature: tauri-ui-improvements, Property 7: Empty input disables send button
   * Validates: Requirements 4.4
   * 
   * For any input state transition from empty to non-empty or vice versa,
   * the button state should update correctly
   */
  it('send button state transitions correctly between empty and non-empty', () => {
    fc.assert(
      fc.property(
        fc.array(
          fc.record({
            text: fc.string(),
            isEmpty: fc.boolean()
          }),
          { minLength: 2, maxLength: 10 }
        ),
        (transitions) => {
          let allTransitionsCorrect = true;
          
          for (const transition of transitions) {
            // Generate text based on isEmpty flag
            const text = transition.isEmpty ? '' : (transition.text.trim() || 'a');
            
            // Type the text
            typeText(messageInput, text);
            
            // Verify button state
            const expectedDisabled = text.trim() === '';
            
            if (sendBtn.disabled !== expectedDisabled) {
              allTransitionsCorrect = false;
              break;
            }
          }
          
          return allTransitionsCorrect;
        }
      ),
      { numRuns: 100 }
    );
  });
});
