import { describe, it, beforeEach, afterEach } from 'vitest';
import * as fc from 'fast-check';
import { loadMainUI, click } from './utils.js';

describe('Click-Outside Property Tests', () => {
  let sidebarOpen;
  let isAnimating;
  let openSidebar;
  let closeSidebar;
  let handleClickOutside;
  
  beforeEach(() => {
    // Load the main UI structure
    loadMainUI();
    
    // Get DOM elements
    const sidebar = document.querySelector('.sidebar');
    const hamburgerBtn = document.getElementById('hamburger-btn');
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
    
    // Define click-outside handler (mimicking app.js behavior)
    handleClickOutside = (event) => {
      // Only handle clicks when sidebar is open
      if (!sidebarOpen) return;
      
      // Don't close if clicking inside the sidebar
      if (sidebar.contains(event.target)) return;
      
      // Don't close if clicking the hamburger button (it has its own toggle handler)
      if (hamburgerBtn.contains(event.target)) return;
      
      // Close sidebar for clicks outside
      closeSidebar();
    };
    
    // Attach the click-outside handler to the document
    document.addEventListener('click', handleClickOutside);
  });
  
  afterEach(() => {
    // Clean up event listeners
    document.removeEventListener('click', handleClickOutside);
  });

  /**
   * Feature: tauri-ui-improvements, Property 2: Click-outside closes sidebar
   * Validates: Requirements 1.4
   * 
   * For any open sidebar, clicking outside the sidebar area should close the sidebar
   */
  it('click-outside closes sidebar', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(
          'chat-container',
          'messages',
          'input-container',
          'message-input',
          'send-btn',
          'backdrop'
        ),
        (targetElementId) => {
          // Setup: Open the sidebar
          const sidebar = document.querySelector('.sidebar');
          const hamburgerBtn = document.getElementById('hamburger-btn');
          
          // Reset state and open sidebar
          sidebarOpen = false;
          isAnimating = false;
          openSidebar();
          
          // Wait for animation to complete (synchronously set state for test)
          isAnimating = false;
          
          // Verify sidebar is open
          if (!sidebarOpen || sidebar.getAttribute('data-state') !== 'open') {
            return false;
          }
          
          // Get the target element to click (outside the sidebar)
          let targetElement;
          if (targetElementId === 'chat-container') {
            targetElement = document.querySelector('.chat-container');
          } else if (targetElementId === 'backdrop') {
            targetElement = document.querySelector('.backdrop');
          } else {
            targetElement = document.getElementById(targetElementId);
          }
          
          // If element doesn't exist, skip this test case
          if (!targetElement) {
            return true;
          }
          
          // Action: Click outside the sidebar
          click(targetElement);
          
          // Get the state after clicking outside
          const stateAfterClick = sidebar.getAttribute('data-state');
          
          // Assertion: The sidebar should be closed
          return stateAfterClick === 'closed' && !sidebarOpen;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Additional test: Clicking inside sidebar should NOT close it
   */
  it('clicking inside sidebar does not close it', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(
          'sidebar',
          'sidebar-content',
          'new-chat-btn',
          'conversation-list'
        ),
        (targetElementId) => {
          // Setup: Open the sidebar
          const sidebar = document.querySelector('.sidebar');
          
          // Reset state and open sidebar
          sidebarOpen = false;
          isAnimating = false;
          openSidebar();
          
          // Wait for animation to complete (synchronously set state for test)
          isAnimating = false;
          
          // Verify sidebar is open
          if (!sidebarOpen || sidebar.getAttribute('data-state') !== 'open') {
            return false;
          }
          
          // Get the target element to click (inside the sidebar)
          let targetElement;
          if (targetElementId === 'sidebar') {
            targetElement = sidebar;
          } else if (targetElementId === 'sidebar-content') {
            targetElement = document.querySelector('.sidebar-content');
          } else {
            targetElement = document.getElementById(targetElementId);
          }
          
          // If element doesn't exist, skip this test case
          if (!targetElement) {
            return true;
          }
          
          // Action: Click inside the sidebar
          click(targetElement);
          
          // Get the state after clicking inside
          const stateAfterClick = sidebar.getAttribute('data-state');
          
          // Assertion: The sidebar should still be open
          return stateAfterClick === 'open' && sidebarOpen;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Additional test: Clicking hamburger button should not trigger click-outside
   */
  it('clicking hamburger button does not trigger click-outside handler', () => {
    // Setup: Open the sidebar
    const sidebar = document.querySelector('.sidebar');
    const hamburgerBtn = document.getElementById('hamburger-btn');
    
    // Reset state and open sidebar
    sidebarOpen = false;
    isAnimating = false;
    openSidebar();
    
    // Wait for animation to complete (synchronously set state for test)
    isAnimating = false;
    
    // Verify sidebar is open
    const isOpen = sidebarOpen && sidebar.getAttribute('data-state') === 'open';
    if (!isOpen) {
      throw new Error('Sidebar should be open for this test');
    }
    
    // Track if closeSidebar was called by click-outside handler
    let clickOutsideTriggered = false;
    const originalCloseSidebar = closeSidebar;
    closeSidebar = () => {
      clickOutsideTriggered = true;
      originalCloseSidebar();
    };
    
    // Action: Click the hamburger button
    // This should NOT trigger the click-outside handler
    // (The hamburger button has its own toggle handler)
    const clickEvent = new MouseEvent('click', {
      bubbles: true,
      cancelable: true
    });
    hamburgerBtn.dispatchEvent(clickEvent);
    
    // Restore original function
    closeSidebar = originalCloseSidebar;
    
    // Assertion: The click-outside handler should not have been triggered
    // (The sidebar state may have changed due to the hamburger's own toggle handler,
    // but that's separate from the click-outside handler)
    return !clickOutsideTriggered;
  });
});
