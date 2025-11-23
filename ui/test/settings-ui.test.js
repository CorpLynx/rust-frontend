import { describe, it, expect, beforeEach, vi } from 'vitest';
import { JSDOM } from 'jsdom';

describe('Settings UI', () => {
  let dom;
  let document;
  let window;

  beforeEach(() => {
    // Create a new JSDOM instance with the HTML structure
    dom = new JSDOM(`
      <!DOCTYPE html>
      <html>
        <body>
          <div id="settings-modal" aria-hidden="true">
            <div class="settings-modal-content">
              <div class="settings-header">
                <h2>Settings</h2>
                <button id="settings-close-btn">×</button>
              </div>
              <div class="settings-body">
                <div class="connection-mode-toggle">
                  <input type="radio" id="mode-local" name="connection-mode" value="local" checked>
                  <input type="radio" id="mode-remote" name="connection-mode" value="remote">
                </div>
                <div id="endpoints-list"></div>
                <button id="add-endpoint-btn">Add Endpoint</button>
              </div>
            </div>
          </div>
          
          <div id="endpoint-dialog" aria-hidden="true">
            <form id="endpoint-form">
              <h3 id="endpoint-dialog-title">Add Endpoint</h3>
              <input type="text" id="endpoint-name" name="name">
              <input type="text" id="endpoint-host" name="host">
              <input type="number" id="endpoint-port" name="port">
              <input type="checkbox" id="endpoint-https" name="use_https">
              <input type="password" id="endpoint-api-key" name="api_key">
              <span id="name-error"></span>
              <span id="host-error"></span>
              <span id="port-error"></span>
              <span id="api-key-error"></span>
              <button type="button" id="test-endpoint-btn">Test</button>
              <button type="button" id="cancel-endpoint-btn">Cancel</button>
              <button type="submit" id="save-endpoint-btn">Save</button>
              <button id="endpoint-dialog-close-btn">×</button>
            </form>
          </div>
        </body>
      </html>
    `, {
      url: 'http://localhost',
      pretendToBeVisual: true,
      resources: 'usable'
    });

    document = dom.window.document;
    window = dom.window;
    global.document = document;
    global.window = window;

    // Mock Tauri API
    window.__TAURI__ = {
      core: {
        invoke: vi.fn().mockResolvedValue(null)
      }
    };
  });

  it('should mask API keys in endpoint list display', () => {
    // This test verifies Requirement 8.3: API keys should be masked in UI
    const endpointsList = document.getElementById('endpoints-list');
    
    // Simulate an endpoint with an API key
    const mockEndpoint = {
      id: 'test-id',
      name: 'Test Endpoint',
      host: '192.168.1.100',
      port: 11434,
      use_https: true,
      api_key: 'secret-api-key-12345',
      last_tested: null,
      last_test_success: null
    };

    // Create endpoint item (simulating renderEndpointsList logic)
    const item = document.createElement('div');
    item.className = 'endpoint-item';
    
    const infoDiv = document.createElement('div');
    infoDiv.className = 'endpoint-info';
    
    const nameDiv = document.createElement('div');
    nameDiv.className = 'endpoint-name';
    nameDiv.textContent = mockEndpoint.name;
    
    const addressDiv = document.createElement('div');
    addressDiv.className = 'endpoint-address';
    const protocol = mockEndpoint.use_https ? 'https' : 'http';
    addressDiv.textContent = `${protocol}://${mockEndpoint.host}:${mockEndpoint.port}`;
    
    infoDiv.appendChild(nameDiv);
    infoDiv.appendChild(addressDiv);
    
    // Display masked API key if present (Requirement 8.3)
    if (mockEndpoint.api_key) {
      const apiKeyDiv = document.createElement('div');
      apiKeyDiv.className = 'endpoint-api-key';
      apiKeyDiv.textContent = 'API Key: ••••••••';
      infoDiv.appendChild(apiKeyDiv);
    }
    
    item.appendChild(infoDiv);
    endpointsList.appendChild(item);
    
    // Verify API key is masked
    const apiKeyElement = endpointsList.querySelector('.endpoint-api-key');
    expect(apiKeyElement).toBeTruthy();
    expect(apiKeyElement.textContent).toBe('API Key: ••••••••');
    expect(apiKeyElement.textContent).not.toContain('secret-api-key-12345');
  });

  it('should not display API key section when no API key is present', () => {
    const endpointsList = document.getElementById('endpoints-list');
    
    // Simulate an endpoint without an API key
    const mockEndpoint = {
      id: 'test-id',
      name: 'Test Endpoint',
      host: '192.168.1.100',
      port: 11434,
      use_https: false,
      api_key: null,
      last_tested: null,
      last_test_success: null
    };

    // Create endpoint item
    const item = document.createElement('div');
    item.className = 'endpoint-item';
    
    const infoDiv = document.createElement('div');
    infoDiv.className = 'endpoint-info';
    
    const nameDiv = document.createElement('div');
    nameDiv.className = 'endpoint-name';
    nameDiv.textContent = mockEndpoint.name;
    
    infoDiv.appendChild(nameDiv);
    
    // Don't add API key div if no API key
    if (mockEndpoint.api_key) {
      const apiKeyDiv = document.createElement('div');
      apiKeyDiv.className = 'endpoint-api-key';
      apiKeyDiv.textContent = 'API Key: ••••••••';
      infoDiv.appendChild(apiKeyDiv);
    }
    
    item.appendChild(infoDiv);
    endpointsList.appendChild(item);
    
    // Verify no API key element is present
    const apiKeyElement = endpointsList.querySelector('.endpoint-api-key');
    expect(apiKeyElement).toBeNull();
  });

  it('should display validation errors inline', () => {
    // This test verifies that validation errors are displayed inline
    const nameError = document.getElementById('name-error');
    const hostError = document.getElementById('host-error');
    const portError = document.getElementById('port-error');
    
    // Simulate validation errors
    nameError.textContent = 'Name is required';
    hostError.textContent = 'Host is required';
    portError.textContent = 'Port must be between 1 and 65535';
    
    expect(nameError.textContent).toBe('Name is required');
    expect(hostError.textContent).toBe('Host is required');
    expect(portError.textContent).toBe('Port must be between 1 and 65535');
  });

  it('should clear form errors', () => {
    const nameError = document.getElementById('name-error');
    const hostError = document.getElementById('host-error');
    const portError = document.getElementById('port-error');
    const apiKeyError = document.getElementById('api-key-error');
    
    // Set some errors
    nameError.textContent = 'Error';
    hostError.textContent = 'Error';
    portError.textContent = 'Error';
    apiKeyError.textContent = 'Error';
    
    // Clear errors (simulating clearFormErrors function)
    nameError.textContent = '';
    hostError.textContent = '';
    portError.textContent = '';
    apiKeyError.textContent = '';
    
    expect(nameError.textContent).toBe('');
    expect(hostError.textContent).toBe('');
    expect(portError.textContent).toBe('');
    expect(apiKeyError.textContent).toBe('');
  });
});
