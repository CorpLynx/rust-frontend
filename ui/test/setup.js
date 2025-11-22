import { beforeEach, afterEach } from 'vitest';

// Mock Tauri API for testing
global.window = global.window || {};
global.window.__TAURI__ = {
  core: {
    invoke: async (cmd, args) => {
      // Mock implementation for testing
      console.log(`Mock Tauri invoke: ${cmd}`, args);
      return null;
    }
  }
};

// Mock localStorage for testing
const localStorageMock = (() => {
  let store = {};
  return {
    getItem: (key) => store[key] || null,
    setItem: (key, value) => {
      store[key] = value.toString();
    },
    removeItem: (key) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    }
  };
})();

global.localStorage = localStorageMock;

// Clean up DOM after each test
afterEach(() => {
  document.body.innerHTML = '';
  document.head.innerHTML = '';
  localStorage.clear();
});

// Reset any global state
beforeEach(() => {
  // Reset any global variables if needed
});
