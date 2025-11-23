/**
 * Tests for UI error handling and feedback (Task 13)
 * Requirements: 4.3, 4.4, 7.1, 7.2, 7.3, 7.4
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { JSDOM } from 'jsdom';
import fs from 'fs';
import path from 'path';

describe('UI Error Handling and Feedback', () => {
    let dom;
    let document;
    let window;

    beforeEach(() => {
        // Load the HTML
        const html = fs.readFileSync(path.resolve(__dirname, '../index.html'), 'utf-8');
        const css = fs.readFileSync(path.resolve(__dirname, '../styles.css'), 'utf-8');
        
        dom = new JSDOM(html, {
            runScripts: 'dangerously',
            resources: 'usable',
            url: 'http://localhost'
        });
        
        document = dom.window.document;
        window = dom.window;
        
        // Add CSS
        const style = document.createElement('style');
        style.textContent = css;
        document.head.appendChild(style);
        
        // Mock Tauri API
        window.__TAURI__ = {
            core: {
                invoke: vi.fn()
            },
            event: {
                listen: vi.fn()
            },
            fs: {
                readTextFile: vi.fn()
            }
        };
        
        // Load the app.js code and execute it
        const appJs = fs.readFileSync(path.resolve(__dirname, '../app.js'), 'utf-8');
        // Remove the init call at the end but keep function definitions
        const scriptWithoutAutoInit = appJs.replace(/if \(document\.readyState === 'loading'\)[\s\S]*?init\(\);?\s*\}/, '');
        
        // Execute the script in the window context
        const scriptFn = new Function('window', 'document', scriptWithoutAutoInit);
        scriptFn(window, document);
    });

    afterEach(() => {
        dom.window.close();
    });

    describe('Toast Notifications', () => {
        it('should display success toast notification', (done) => {
            const { showToast } = window;
            
            showToast('Test success message', 'success');
            
            setTimeout(() => {
                const toast = document.querySelector('.toast.toast-success');
                expect(toast).toBeTruthy();
                expect(toast.textContent).toBe('Test success message');
                expect(toast.classList.contains('show')).toBe(true);
                done();
            }, 50);
        });

        it('should display error toast notification', (done) => {
            const { showToast } = window;
            
            showToast('Test error message', 'error');
            
            setTimeout(() => {
                const toast = document.querySelector('.toast.toast-error');
                expect(toast).toBeTruthy();
                expect(toast.textContent).toBe('Test error message');
                done();
            }, 50);
        });

        it('should auto-remove toast after duration', (done) => {
            const { showToast } = window;
            
            showToast('Test message', 'info', 100);
            
            setTimeout(() => {
                const toast = document.querySelector('.toast');
                expect(toast).toBeTruthy();
            }, 50);
            
            setTimeout(() => {
                const toast = document.querySelector('.toast');
                expect(toast).toBeFalsy();
                done();
            }, 500);
        });
    });

    describe('Error Message Formatting', () => {
        it('should format timeout errors with helpful message (Requirement 7.1)', () => {
            const { formatErrorMessage } = window;
            
            const result = formatErrorMessage('Connection timeout');
            expect(result).toContain('Server unreachable');
            expect(result).toContain('timed out');
        });

        it('should format connection refused errors (Requirement 7.2)', () => {
            const { formatErrorMessage } = window;
            
            const result = formatErrorMessage('Connection refused');
            expect(result).toContain('not accepting connections');
            expect(result).toContain('server is running');
        });

        it('should format invalid response errors (Requirement 7.3)', () => {
            const { formatErrorMessage } = window;
            
            const result = formatErrorMessage('Invalid response from server');
            expect(result).toContain('protocol mismatch');
            expect(result).toContain('Ollama server');
        });

        it('should format TLS/SSL errors with helpful message (Requirement 7.4)', () => {
            const { formatErrorMessage } = window;
            
            const result = formatErrorMessage('TLS certificate error');
            expect(result).toContain('certificate');
            expect(result).toContain('HTTP instead');
        });

        it('should format DNS resolution errors', () => {
            const { formatErrorMessage } = window;
            
            const result = formatErrorMessage('DNS resolution failed');
            expect(result).toContain('Cannot resolve hostname');
        });
    });

    describe('Connection Test Result Display', () => {
        it('should display successful connection test result (Requirement 4.3)', () => {
            const { showConnectionTestResult } = window;
            
            const result = {
                success: true,
                response_time_ms: 150
            };
            
            const container = document.createElement('div');
            document.body.appendChild(container);
            
            showConnectionTestResult(result, container);
            
            const resultDiv = container.querySelector('.connection-test-result');
            expect(resultDiv).toBeTruthy();
            expect(resultDiv.classList.contains('success')).toBe(true);
            expect(resultDiv.textContent).toContain('Connection successful');
            expect(resultDiv.textContent).toContain('150ms');
        });

        it('should display failed connection test result (Requirement 4.4)', () => {
            const { showConnectionTestResult } = window;
            
            const result = {
                success: false,
                error_message: 'Connection timeout'
            };
            
            const container = document.createElement('div');
            document.body.appendChild(container);
            
            showConnectionTestResult(result, container);
            
            const resultDiv = container.querySelector('.connection-test-result');
            expect(resultDiv).toBeTruthy();
            expect(resultDiv.classList.contains('error')).toBe(true);
            expect(resultDiv.textContent).toContain('Connection failed');
            expect(resultDiv.textContent).toContain('Server unreachable');
        });

        it('should replace existing test result in container', () => {
            const { showConnectionTestResult } = window;
            
            const container = document.createElement('div');
            document.body.appendChild(container);
            
            // Add first result
            showConnectionTestResult({ success: true, response_time_ms: 100 }, container);
            expect(container.querySelectorAll('.connection-test-result').length).toBe(1);
            
            // Add second result - should replace first
            showConnectionTestResult({ success: false, error_message: 'Error' }, container);
            expect(container.querySelectorAll('.connection-test-result').length).toBe(1);
            expect(container.querySelector('.connection-test-result').classList.contains('error')).toBe(true);
        });
    });

    describe('Loading State Manager', () => {
        it('should track loading operations', () => {
            const { loadingManager } = window;
            
            expect(loadingManager.isLoading()).toBe(false);
            
            loadingManager.startOperation('test-op');
            expect(loadingManager.isLoading()).toBe(true);
            expect(loadingManager.isLoading('test-op')).toBe(true);
            
            loadingManager.endOperation('test-op');
            expect(loadingManager.isLoading()).toBe(false);
        });

        it('should add loading class to element', () => {
            const { loadingManager } = window;
            
            const button = document.createElement('button');
            button.disabled = false;
            
            loadingManager.startOperation('test', button);
            expect(button.classList.contains('loading')).toBe(true);
            expect(button.disabled).toBe(true);
            
            loadingManager.endOperation('test', button);
            expect(button.classList.contains('loading')).toBe(false);
            expect(button.disabled).toBe(false);
        });

        it('should handle multiple concurrent operations', () => {
            const { loadingManager } = window;
            
            loadingManager.startOperation('op1');
            loadingManager.startOperation('op2');
            expect(loadingManager.isLoading()).toBe(true);
            
            loadingManager.endOperation('op1');
            expect(loadingManager.isLoading()).toBe(true);
            
            loadingManager.endOperation('op2');
            expect(loadingManager.isLoading()).toBe(false);
        });
    });

    describe('Inline Validation Errors', () => {
        it('should display inline validation error for empty name', () => {
            const { validateEndpointName } = window;
            
            const nameInput = document.getElementById('endpoint-name');
            const nameError = document.getElementById('name-error');
            
            nameInput.value = '';
            const isValid = validateEndpointName();
            
            expect(isValid).toBe(false);
            expect(nameError.textContent).toContain('required');
        });

        it('should display inline validation error for invalid host', () => {
            const { validateEndpointHost } = window;
            
            const hostInput = document.getElementById('endpoint-host');
            const hostError = document.getElementById('host-error');
            
            hostInput.value = 'invalid..host';
            const isValid = validateEndpointHost();
            
            expect(isValid).toBe(false);
            expect(hostError.textContent).toContain('Invalid');
        });

        it('should display inline validation error for invalid port', () => {
            const { validateEndpointPort } = window;
            
            const portInput = document.getElementById('endpoint-port');
            const portError = document.getElementById('port-error');
            
            portInput.value = '99999';
            const isValid = validateEndpointPort();
            
            expect(isValid).toBe(false);
            expect(portError.textContent).toContain('between 1 and 65535');
        });

        it('should clear validation error when input becomes valid', () => {
            const { validateEndpointName } = window;
            
            const nameInput = document.getElementById('endpoint-name');
            const nameError = document.getElementById('name-error');
            
            nameInput.value = '';
            validateEndpointName();
            expect(nameError.textContent).toBeTruthy();
            
            nameInput.value = 'Valid Name';
            validateEndpointName();
            expect(nameError.textContent).toBe('');
        });
    });
});
