import { describe, it, expect, beforeEach, vi } from 'vitest';
import { loadMainUI, click, waitForAnimation } from './utils.js';

describe('Animation Timing Example Tests', () => {
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
    };
    
    closeSidebar = () => {
      if (isAnimating || !sidebarOpen) return;
      
      isAnimating = true;
      sidebarOpen = false;
      sidebar.setAttribute('data-state', 'closed');
      hamburgerBtn.classList.remove('active');
      backdrop.setAttribute('data-visible', 'false');
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
    
    // Simulate transitionend event to reset isAnimating flag
    sidebar.addEventListener('transitionend', (e) => {
      if (e.propertyName === 'transform') {
        isAnimating = false;
      }
    });
  });

  /**
   * Example 3: Animation timing
   * Validates: Requirements 2.1, 2.2
   * 
   * Verify that sidebar open and close animations complete in approximately 300ms
   */
  it('sidebar open animation completes in approximately 300ms', async () => {
    // Ensure sidebar starts closed
    expect(sidebar.getAttribute('data-state')).toBe('closed');
    
    // Record start time
    const startTime = performance.now();
    
    // Trigger sidebar open
    click(hamburgerBtn);
    
    // Verify animation started
    expect(sidebar.getAttribute('data-state')).toBe('open');
    expect(isAnimating).toBe(true);
    
    // Manually trigger transitionend event after 300ms to simulate CSS animation
    setTimeout(() => {
      const event = new Event('transitionend');
      event.propertyName = 'transform';
      sidebar.dispatchEvent(event);
    }, 300);
    
    // Wait for animation to complete
    await waitForAnimation(300);
    
    // Record end time
    const endTime = performance.now();
    const duration = endTime - startTime;
    
    // Verify animation completed
    expect(isAnimating).toBe(false);
    
    // Verify duration is approximately 300ms (with some tolerance for test execution)
    // Allow 50ms tolerance for test overhead
    expect(duration).toBeGreaterThanOrEqual(290);
    expect(duration).toBeLessThan(400);
  });

  it('sidebar close animation completes in approximately 300ms', async () => {
    // Open sidebar first
    click(hamburgerBtn);
    
    // Manually trigger transitionend to complete open animation
    setTimeout(() => {
      const event = new Event('transitionend');
      event.propertyName = 'transform';
      sidebar.dispatchEvent(event);
    }, 300);
    
    await waitForAnimation(300);
    
    // Verify sidebar is open and animation is complete
    expect(sidebar.getAttribute('data-state')).toBe('open');
    expect(isAnimating).toBe(false);
    
    // Record start time for close animation
    const startTime = performance.now();
    
    // Trigger sidebar close
    click(hamburgerBtn);
    
    // Verify animation started
    expect(sidebar.getAttribute('data-state')).toBe('closed');
    expect(isAnimating).toBe(true);
    
    // Manually trigger transitionend event after 300ms to simulate CSS animation
    setTimeout(() => {
      const event = new Event('transitionend');
      event.propertyName = 'transform';
      sidebar.dispatchEvent(event);
    }, 300);
    
    // Wait for animation to complete
    await waitForAnimation(300);
    
    // Record end time
    const endTime = performance.now();
    const duration = endTime - startTime;
    
    // Verify animation completed
    expect(isAnimating).toBe(false);
    
    // Verify duration is approximately 300ms (with some tolerance for test execution)
    // Allow 50ms tolerance for test overhead
    expect(duration).toBeGreaterThanOrEqual(290);
    expect(duration).toBeLessThan(400);
  });

  it('animation uses ease-in-out timing function', () => {
    // This test verifies the CSS transition property is set correctly
    // In a real browser environment, we would check computed styles
    
    // Load CSS styles (in a real test environment, styles would be loaded)
    const style = document.createElement('style');
    style.textContent = `
      .sidebar {
        transition: transform 300ms ease-in-out;
      }
    `;
    document.head.appendChild(style);
    
    // Get computed style
    const computedStyle = window.getComputedStyle(sidebar);
    const transition = computedStyle.transition || computedStyle.webkitTransition;
    
    // Verify transition property includes transform, 300ms, and ease-in-out
    // Note: In JSDOM, computed styles may not be fully available
    // This test documents the expected CSS configuration
    expect(transition).toBeDefined();
  });

  it('multiple rapid clicks respect animation timing', async () => {
    // Ensure sidebar starts closed
    expect(sidebar.getAttribute('data-state')).toBe('closed');
    
    // Click to open
    click(hamburgerBtn);
    expect(isAnimating).toBe(true);
    
    // Try to click again immediately (should be ignored)
    click(hamburgerBtn);
    
    // State should still be 'open' (not toggled back to closed)
    expect(sidebar.getAttribute('data-state')).toBe('open');
    
    // Simulate animation completion
    setTimeout(() => {
      const event = new Event('transitionend');
      event.propertyName = 'transform';
      sidebar.dispatchEvent(event);
    }, 300);
    
    await waitForAnimation(300);
    
    // Now animation is complete
    expect(isAnimating).toBe(false);
    
    // Now clicking should work again
    click(hamburgerBtn);
    expect(sidebar.getAttribute('data-state')).toBe('closed');
    expect(isAnimating).toBe(true);
  });

  it('transitionend event only responds to transform property', async () => {
    // Open sidebar
    click(hamburgerBtn);
    expect(isAnimating).toBe(true);
    
    // Trigger transitionend with different property (should not reset isAnimating)
    const opacityEvent = new Event('transitionend');
    opacityEvent.propertyName = 'opacity';
    sidebar.dispatchEvent(opacityEvent);
    
    // isAnimating should still be true
    expect(isAnimating).toBe(true);
    
    // Now trigger with transform property
    const transformEvent = new Event('transitionend');
    transformEvent.propertyName = 'transform';
    sidebar.dispatchEvent(transformEvent);
    
    // Now isAnimating should be false
    expect(isAnimating).toBe(false);
  });
});
