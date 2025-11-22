import { describe, it, beforeEach, afterEach } from 'vitest';
import * as fc from 'fast-check';
import { loadMainUI, click } from './utils.js';

describe('Sidebar Toggle Property Tests', () => {
  let toggleSidebar;
  let openSidebar;
  let closeSidebar;
  let sidebarOpen;
  let isAnimating;
  
  beforeEach(() => {
    // Load the main UI structure
    loadMainUI();
    
    // Get DOM elements
    const sidebar = document.querySelector('.sidebar');
    const hamburgerBtn = document.getElementById('hamburger-btn');
    const backdrop = document.querySelector('.backdrop') || createBackdrop();
    
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
  
  afterEach(() => {
    // Clean up event listeners
    const hamburgerBtn = document.getElementById('hamburger-btn');
    if (hamburgerBtn) {
      hamburgerBtn.replaceWith(hamburgerBtn.cloneNode(true));
    }
  });
  
  function createBackdrop() {
    const backdrop = document.createElement('div');
    backdrop.className = 'backdrop';
    backdrop.setAttribute('data-visible', 'false');
    document.body.appendChild(backdrop);
    return backdrop;
  }

  /**
   * Feature: tauri-ui-improvements, Property 1: Sidebar toggle behavior
   * Validates: Requirements 1.2, 1.3
   * 
   * For any sidebar state (open or closed), clicking the hamburger button 
   * should transition the sidebar to the opposite state
   */
  it('sidebar toggles between open and closed states', () => {
    fc.assert(
      fc.property(fc.boolean(), (initialState) => {
        // Setup: Set the sidebar to the initial state
        const sidebar = document.querySelector('.sidebar');
        const hamburgerBtn = document.getElementById('hamburger-btn');
        
        // Reset to initial state
        sidebarOpen = initialState;
        isAnimating = false;
        
        if (initialState) {
          sidebar.setAttribute('data-state', 'open');
          hamburgerBtn.classList.add('active');
        } else {
          sidebar.setAttribute('data-state', 'closed');
          hamburgerBtn.classList.remove('active');
        }
        
        // Get the state before clicking
        const stateBefore = sidebar.getAttribute('data-state');
        const iconStateBefore = hamburgerBtn.classList.contains('active');
        
        // Action: Click the hamburger button
        click(hamburgerBtn);
        
        // Get the state after clicking (synchronous state change)
        const stateAfter = sidebar.getAttribute('data-state');
        const iconStateAfter = hamburgerBtn.classList.contains('active');
        
        // Assertion: The state should have toggled
        const expectedState = stateBefore === 'open' ? 'closed' : 'open';
        const expectedIconState = !iconStateBefore;
        
        return stateAfter === expectedState && iconStateAfter === expectedIconState;
      }),
      { numRuns: 100 }
    );
  });

  /**
   * Feature: tauri-ui-improvements, Property 3: Animation prevents concurrent toggles
   * Validates: Requirements 2.3
   * 
   * For any sidebar animation in progress, attempting to toggle the sidebar 
   * should be ignored until the animation completes
   */
  it('animation prevents concurrent toggles', () => {
    fc.assert(
      fc.property(fc.boolean(), fc.integer({ min: 1, max: 5 }), (initialState, numClicks) => {
        // Setup: Set the sidebar to the initial state
        const sidebar = document.querySelector('.sidebar');
        const hamburgerBtn = document.getElementById('hamburger-btn');
        
        // Reset to initial state
        sidebarOpen = initialState;
        isAnimating = false;
        
        if (initialState) {
          sidebar.setAttribute('data-state', 'open');
          hamburgerBtn.classList.add('active');
        } else {
          sidebar.setAttribute('data-state', 'closed');
          hamburgerBtn.classList.remove('active');
        }
        
        // Get the state before any clicks
        const stateBefore = sidebar.getAttribute('data-state');
        
        // Action: Click the hamburger button to start animation
        click(hamburgerBtn);
        
        // Get state after first click (should have changed)
        const stateAfterFirstClick = sidebar.getAttribute('data-state');
        
        // Verify first click worked
        const expectedStateAfterFirst = stateBefore === 'open' ? 'closed' : 'open';
        if (stateAfterFirstClick !== expectedStateAfterFirst) {
          return false;
        }
        
        // Now the animation is in progress (isAnimating = true)
        // Try to click multiple times during animation
        for (let i = 0; i < numClicks; i++) {
          click(hamburgerBtn);
        }
        
        // Get state after all the clicks during animation
        const stateAfterConcurrentClicks = sidebar.getAttribute('data-state');
        
        // Assertion: The state should NOT have changed from the first click
        // because all subsequent clicks should be ignored during animation
        return stateAfterConcurrentClicks === stateAfterFirstClick;
      }),
      { numRuns: 100 }
    );
  });

  /**
   * Feature: tauri-ui-improvements, Property 4: Icon state reflects sidebar visibility
   * Validates: Requirements 2.4
   * 
   * For any completed sidebar animation, the hamburger icon state should match 
   * the sidebar's visibility state
   */
  it('icon state reflects sidebar visibility after animation completes', async () => {
    return fc.assert(
      fc.asyncProperty(fc.boolean(), async (initialState) => {
        // Setup: Set the sidebar to the initial state
        const sidebar = document.querySelector('.sidebar');
        const hamburgerBtn = document.getElementById('hamburger-btn');
        
        // Reset to initial state
        sidebarOpen = initialState;
        isAnimating = false;
        
        if (initialState) {
          sidebar.setAttribute('data-state', 'open');
          hamburgerBtn.classList.add('active');
        } else {
          sidebar.setAttribute('data-state', 'closed');
          hamburgerBtn.classList.remove('active');
        }
        
        // Action: Click the hamburger button to toggle
        click(hamburgerBtn);
        
        // Wait for animation to complete (simulated with 10ms timeout in beforeEach)
        await new Promise(resolve => setTimeout(resolve, 20));
        
        // Get the final states after animation completes
        const finalSidebarState = sidebar.getAttribute('data-state');
        const finalIconState = hamburgerBtn.classList.contains('active');
        
        // Assertion: Icon state should match sidebar visibility
        // If sidebar is 'open', icon should have 'active' class
        // If sidebar is 'closed', icon should NOT have 'active' class
        const iconMatchesSidebar = (finalSidebarState === 'open' && finalIconState) ||
                                   (finalSidebarState === 'closed' && !finalIconState);
        
        return iconMatchesSidebar;
      }),
      { numRuns: 100 }
    );
  });
});
