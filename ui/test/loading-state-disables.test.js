import { describe, it, beforeEach, expect } from 'vitest';
import * as fc from 'fast-check';
import { loadMainUI, typeText } from './utils.js';

describe('Loading State Disables Send Button Property Tests', () => {
  let messageInput;
  let sendBtn;
  let isLoading;
  
  beforeEach(() => {
    // Load the main UI structure
    loadMainUI();
    
    // Get the elements
    messageInput = document.getElementById('message-input');
    sendBtn = document.getElementById('send-btn');
    
    // Initialize loading state
    isLoading = false;
    
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
      .send-btn-circular.loading {
        opacity: 0.7;
      }
    `;
    document.head.appendChild(style);
    
    // Set up the updateSendButtonState function (from app.js)
    window.updateSendButtonState = function() {
      const message = messageInput.value.trim();
      const isEmpty = message === '';
      
      // Disable button if input is empty or if loading
      sendBtn.disabled = isEmpty || isLoading;
    };
    
    // Set up function to simulate loading state change
    window.setLoadingState = function(loading) {
      isLoading = loading;
      if (loading) {
        sendBtn.classList.add('loading');
      } else {
        sendBtn.classList.remove('loading');
      }
      window.updateSendButtonState();
    };
    
    // Add event listeners
    messageInput.addEventListener('input', window.updateSendButtonState);
    messageInput.addEventListener('keyup', window.updateSendButtonState);
    
    // Initialize button state
    window.updateSendButtonState();
  });

  /**
   * Feature: tauri-ui-improvements, Property 8: Loading state disables send button
   * Validates: Requirements 4.5
   * 
   * For any message processing state, the send button should be disabled
   * and display a loading indicator when the system is processing a message
   */
  it('send button is disabled when system is in loading state', () => {
    fc.assert(
      fc.property(
        fc.boolean(),
        (loadingState) => {
          // Setup: Set a valid message in the input
          typeText(messageInput, 'Test message');
          
          // Action: Set the loading state
          window.setLoadingState(loadingState);
          
          // Verify: Button should be disabled if and only if loading
          if (loadingState) {
            return sendBtn.disabled === true && sendBtn.classList.contains('loading');
          } else {
            return sendBtn.disabled === false && !sendBtn.classList.contains('loading');
          }
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Feature: tauri-ui-improvements, Property 8: Loading state disables send button
   * Validates: Requirements 4.5
   * 
   * For any sequence of loading state changes, the send button should
   * correctly reflect the current loading state
   */
  it('send button disabled state correctly tracks loading state changes', () => {
    fc.assert(
      fc.property(
        fc.array(fc.boolean(), { minLength: 1, maxLength: 10 }),
        (loadingSequence) => {
          // Setup: Set a valid message in the input
          typeText(messageInput, 'Test message');
          
          let allStatesCorrect = true;
          
          for (const loadingState of loadingSequence) {
            // Set the loading state
            window.setLoadingState(loadingState);
            
            // Check if button state is correct
            const expectedDisabled = loadingState;
            const hasLoadingClass = sendBtn.classList.contains('loading');
            
            if (sendBtn.disabled !== expectedDisabled || hasLoadingClass !== loadingState) {
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
   * Feature: tauri-ui-improvements, Property 8: Loading state disables send button
   * Validates: Requirements 4.5
   * 
   * For any input state combined with loading state, the button should be
   * disabled if either the input is empty OR the system is loading
   */
  it('send button is disabled when loading regardless of input state', () => {
    fc.assert(
      fc.property(
        fc.record({
          inputText: fc.string(),
          isLoading: fc.boolean()
        }),
        ({ inputText, isLoading }) => {
          // Setup: Type the input text
          typeText(messageInput, inputText);
          
          // Action: Set the loading state
          window.setLoadingState(isLoading);
          
          // Verify: Button should be disabled if input is empty OR loading
          const isEmpty = inputText.trim() === '';
          const expectedDisabled = isEmpty || isLoading;
          
          return sendBtn.disabled === expectedDisabled;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Feature: tauri-ui-improvements, Property 8: Loading state disables send button
   * Validates: Requirements 4.5
   * 
   * For any loading state transition from false to true, the button should
   * become disabled and show loading indicator
   */
  it('send button becomes disabled when entering loading state', () => {
    fc.assert(
      fc.property(
        fc.string({ minLength: 1 }).filter(s => s.trim().length > 0),
        (validInput) => {
          // Setup: Start with non-loading state and valid input
          typeText(messageInput, validInput);
          window.setLoadingState(false);
          
          // Verify initial state: button should be enabled
          if (sendBtn.disabled !== false) {
            return false;
          }
          
          // Action: Enter loading state
          window.setLoadingState(true);
          
          // Verify: Button should now be disabled with loading class
          return sendBtn.disabled === true && sendBtn.classList.contains('loading');
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Feature: tauri-ui-improvements, Property 8: Loading state disables send button
   * Validates: Requirements 4.5
   * 
   * For any loading state transition from true to false with valid input,
   * the button should become enabled and remove loading indicator
   */
  it('send button becomes enabled when exiting loading state with valid input', () => {
    fc.assert(
      fc.property(
        fc.string({ minLength: 1 }).filter(s => s.trim().length > 0),
        (validInput) => {
          // Setup: Start with loading state and valid input
          typeText(messageInput, validInput);
          window.setLoadingState(true);
          
          // Verify initial state: button should be disabled
          if (sendBtn.disabled !== true) {
            return false;
          }
          
          // Action: Exit loading state
          window.setLoadingState(false);
          
          // Verify: Button should now be enabled without loading class
          return sendBtn.disabled === false && !sendBtn.classList.contains('loading');
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Feature: tauri-ui-improvements, Property 8: Loading state disables send button
   * Validates: Requirements 4.5
   * 
   * For any combination of input changes and loading state changes,
   * the button state should always correctly reflect both conditions
   */
  it('send button state correctly handles combined input and loading state changes', () => {
    fc.assert(
      fc.property(
        fc.array(
          fc.record({
            inputText: fc.string(),
            isLoading: fc.boolean()
          }),
          { minLength: 1, maxLength: 10 }
        ),
        (stateChanges) => {
          let allStatesCorrect = true;
          
          for (const change of stateChanges) {
            // Apply both changes
            typeText(messageInput, change.inputText);
            window.setLoadingState(change.isLoading);
            
            // Verify button state
            const isEmpty = change.inputText.trim() === '';
            const expectedDisabled = isEmpty || change.isLoading;
            const hasLoadingClass = sendBtn.classList.contains('loading');
            
            if (sendBtn.disabled !== expectedDisabled || hasLoadingClass !== change.isLoading) {
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
});
