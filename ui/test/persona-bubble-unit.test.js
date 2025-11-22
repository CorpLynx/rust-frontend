import { describe, it, expect, beforeEach, vi } from 'vitest';
import { loadHTML } from './utils.js';

// PersonaBubble component for testing
class PersonaBubble {
    constructor(persona) {
        this.persona = persona;
        this.element = null;
        this.isActive = false;
        this.clickCallback = null;
    }
    
    render() {
        const button = document.createElement('button');
        button.className = 'persona-bubble';
        button.textContent = this.persona.icon;
        button.setAttribute('aria-label', `Select ${this.persona.name} persona`);
        button.setAttribute('aria-pressed', 'false');
        button.setAttribute('role', 'button');
        button.setAttribute('title', `${this.persona.name}${this.persona.description ? ': ' + this.persona.description : ''}`);
        button.setAttribute('tabindex', '0');
        button.dataset.personaId = this.persona.id;
        
        button.addEventListener('click', () => {
            if (this.clickCallback) {
                this.clickCallback(this.persona.id);
            }
        });
        
        button.addEventListener('keydown', (e) => {
            if (e.key === ' ' || e.key === 'Enter') {
                e.preventDefault();
                if (this.clickCallback) {
                    this.clickCallback(this.persona.id);
                }
            }
        });
        
        this.element = button;
        return button;
    }
    
    setActive(isActive) {
        this.isActive = isActive;
        if (this.element) {
            if (isActive) {
                this.element.classList.add('active');
                this.element.setAttribute('aria-pressed', 'true');
            } else {
                this.element.classList.remove('active');
                this.element.setAttribute('aria-pressed', 'false');
            }
        }
    }
    
    onClick(callback) {
        this.clickCallback = callback;
    }
    
    setDisabled(disabled) {
        if (this.element) {
            this.element.disabled = disabled;
        }
    }
}

describe('PersonaBubble Unit Tests', () => {
    let container;
    
    beforeEach(() => {
        loadHTML('<div id="test-container"></div>');
        container = document.getElementById('test-container');
    });
    
    describe('Rendering with different persona data', () => {
        it('should render with minimal persona data', () => {
            const persona = {
                id: 'test-1',
                name: 'Test Persona',
                icon: '🤖',
                system_prompt: 'You are a test assistant'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            container.appendChild(element);
            
            expect(element.tagName).toBe('BUTTON');
            expect(element.className).toBe('persona-bubble');
            expect(element.textContent).toBe('🤖');
            expect(element.dataset.personaId).toBe('test-1');
        });
        
        it('should render with full persona data including description', () => {
            const persona = {
                id: 'test-2',
                name: 'Creative Writer',
                icon: '✍️',
                system_prompt: 'You are a creative writing assistant',
                description: 'Helps with creative writing tasks'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            container.appendChild(element);
            
            expect(element.textContent).toBe('✍️');
            expect(element.getAttribute('title')).toBe('Creative Writer: Helps with creative writing tasks');
        });
        
        it('should render without description', () => {
            const persona = {
                id: 'test-3',
                name: 'Code Helper',
                icon: '💻',
                system_prompt: 'You are a coding assistant'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            container.appendChild(element);
            
            expect(element.getAttribute('title')).toBe('Code Helper');
        });
        
        it('should render with different icon types', () => {
            const personas = [
                { id: '1', name: 'Emoji', icon: '🎨', system_prompt: 'test' },
                { id: '2', name: 'Letter', icon: 'A', system_prompt: 'test' },
                { id: '3', name: 'Symbol', icon: '★', system_prompt: 'test' }
            ];
            
            personas.forEach(persona => {
                const bubble = new PersonaBubble(persona);
                const element = bubble.render();
                expect(element.textContent).toBe(persona.icon);
            });
        });
        
        it('should set correct ARIA attributes', () => {
            const persona = {
                id: 'test-4',
                name: 'Test Assistant',
                icon: '🤖',
                system_prompt: 'You are helpful'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            
            expect(element.getAttribute('role')).toBe('button');
            expect(element.getAttribute('aria-label')).toBe('Select Test Assistant persona');
            expect(element.getAttribute('aria-pressed')).toBe('false');
            expect(element.getAttribute('tabindex')).toBe('0');
        });
    });
    
    describe('Click event handling', () => {
        it('should trigger callback when clicked', () => {
            const persona = {
                id: 'click-test',
                name: 'Click Test',
                icon: '👆',
                system_prompt: 'test'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            container.appendChild(element);
            
            const callback = vi.fn();
            bubble.onClick(callback);
            
            element.click();
            
            expect(callback).toHaveBeenCalledTimes(1);
            expect(callback).toHaveBeenCalledWith('click-test');
        });
        
        it('should not error if clicked without callback', () => {
            const persona = {
                id: 'no-callback',
                name: 'No Callback',
                icon: '🚫',
                system_prompt: 'test'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            container.appendChild(element);
            
            expect(() => element.click()).not.toThrow();
        });
        
        it('should handle multiple clicks', () => {
            const persona = {
                id: 'multi-click',
                name: 'Multi Click',
                icon: '🔄',
                system_prompt: 'test'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            container.appendChild(element);
            
            const callback = vi.fn();
            bubble.onClick(callback);
            
            element.click();
            element.click();
            element.click();
            
            expect(callback).toHaveBeenCalledTimes(3);
        });
        
        it('should trigger callback on keyboard Space key', () => {
            const persona = {
                id: 'keyboard-space',
                name: 'Keyboard Space',
                icon: '⌨️',
                system_prompt: 'test'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            container.appendChild(element);
            
            const callback = vi.fn();
            bubble.onClick(callback);
            
            const event = new KeyboardEvent('keydown', { key: ' ' });
            element.dispatchEvent(event);
            
            expect(callback).toHaveBeenCalledTimes(1);
            expect(callback).toHaveBeenCalledWith('keyboard-space');
        });
        
        it('should trigger callback on keyboard Enter key', () => {
            const persona = {
                id: 'keyboard-enter',
                name: 'Keyboard Enter',
                icon: '↵',
                system_prompt: 'test'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            container.appendChild(element);
            
            const callback = vi.fn();
            bubble.onClick(callback);
            
            const event = new KeyboardEvent('keydown', { key: 'Enter' });
            element.dispatchEvent(event);
            
            expect(callback).toHaveBeenCalledTimes(1);
        });
    });
    
    describe('Visual state changes', () => {
        it('should add active class when setActive(true)', () => {
            const persona = {
                id: 'state-test',
                name: 'State Test',
                icon: '🎯',
                system_prompt: 'test'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            container.appendChild(element);
            
            bubble.setActive(true);
            
            expect(bubble.isActive).toBe(true);
            expect(element.classList.contains('active')).toBe(true);
            expect(element.getAttribute('aria-pressed')).toBe('true');
        });
        
        it('should remove active class when setActive(false)', () => {
            const persona = {
                id: 'state-test-2',
                name: 'State Test 2',
                icon: '🎯',
                system_prompt: 'test'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            container.appendChild(element);
            
            bubble.setActive(true);
            bubble.setActive(false);
            
            expect(bubble.isActive).toBe(false);
            expect(element.classList.contains('active')).toBe(false);
            expect(element.getAttribute('aria-pressed')).toBe('false');
        });
        
        it('should toggle active state correctly', () => {
            const persona = {
                id: 'toggle-test',
                name: 'Toggle Test',
                icon: '🔀',
                system_prompt: 'test'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            container.appendChild(element);
            
            // Start inactive
            expect(bubble.isActive).toBe(false);
            
            // Activate
            bubble.setActive(true);
            expect(bubble.isActive).toBe(true);
            expect(element.classList.contains('active')).toBe(true);
            
            // Deactivate
            bubble.setActive(false);
            expect(bubble.isActive).toBe(false);
            expect(element.classList.contains('active')).toBe(false);
            
            // Activate again
            bubble.setActive(true);
            expect(bubble.isActive).toBe(true);
            expect(element.classList.contains('active')).toBe(true);
        });
        
        it('should handle disabled state', () => {
            const persona = {
                id: 'disabled-test',
                name: 'Disabled Test',
                icon: '🚫',
                system_prompt: 'test'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            container.appendChild(element);
            
            bubble.setDisabled(true);
            expect(element.disabled).toBe(true);
            
            bubble.setDisabled(false);
            expect(element.disabled).toBe(false);
        });
        
        it('should maintain visual state when disabled', () => {
            const persona = {
                id: 'disabled-active',
                name: 'Disabled Active',
                icon: '⛔',
                system_prompt: 'test'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            container.appendChild(element);
            
            bubble.setActive(true);
            bubble.setDisabled(true);
            
            expect(element.classList.contains('active')).toBe(true);
            expect(element.disabled).toBe(true);
        });
    });
    
    describe('Edge cases', () => {
        it('should handle setActive before render', () => {
            const persona = {
                id: 'pre-render',
                name: 'Pre Render',
                icon: '⏰',
                system_prompt: 'test'
            };
            
            const bubble = new PersonaBubble(persona);
            
            // Set active before rendering
            bubble.setActive(true);
            expect(bubble.isActive).toBe(true);
            
            // Now render
            const element = bubble.render();
            container.appendChild(element);
            
            // Element should not have active class since it wasn't rendered yet
            // This tests that setActive handles null element gracefully
            expect(element.classList.contains('active')).toBe(false);
        });
        
        it('should handle setDisabled before render', () => {
            const persona = {
                id: 'pre-render-disabled',
                name: 'Pre Render Disabled',
                icon: '⏰',
                system_prompt: 'test'
            };
            
            const bubble = new PersonaBubble(persona);
            
            // Set disabled before rendering - should not error
            expect(() => bubble.setDisabled(true)).not.toThrow();
            
            const element = bubble.render();
            expect(element.disabled).toBe(false); // Not disabled since element didn't exist
        });
        
        it('should handle empty icon gracefully', () => {
            const persona = {
                id: 'empty-icon',
                name: 'Empty Icon',
                icon: '',
                system_prompt: 'test'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            
            expect(element.textContent).toBe('');
        });
        
        it('should handle long persona names', () => {
            const persona = {
                id: 'long-name',
                name: 'This is a very long persona name that might cause layout issues',
                icon: '📏',
                system_prompt: 'test'
            };
            
            const bubble = new PersonaBubble(persona);
            const element = bubble.render();
            
            expect(element.getAttribute('aria-label')).toContain('This is a very long persona name');
        });
    });
});
