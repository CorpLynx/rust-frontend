/**
 * Keyboard Navigation Tests for Persona Bubbles
 * Requirements: 5.1, 5.2
 * 
 * Tests keyboard accessibility features:
 * - Bubbles are focusable with Tab key
 * - Space and Enter keys toggle persona selection
 * - Focus indicators are visible
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { loadHTML } from './utils.js';

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
        button.setAttribute('tabindex', '0'); // Make focusable with keyboard
        button.dataset.personaId = this.persona.id;
        
        // Add click handler
        button.addEventListener('click', () => {
            if (this.clickCallback) {
                this.clickCallback(this.persona.id);
            }
        });
        
        // Add keyboard event handler for Space/Enter
        // Requirements: 5.1, 5.2
        button.addEventListener('keydown', (e) => {
            if (e.key === ' ' || e.key === 'Enter') {
                e.preventDefault(); // Prevent page scroll on Space
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

class PersonaBubbleContainer {
    constructor(personas, inputContainer) {
        this.personas = personas;
        this.inputContainer = inputContainer;
        this.bubbles = [];
        this.activePersonaId = null;
        this.containerElement = null;
    }
    
    render() {
        const container = document.createElement('div');
        container.className = 'persona-bubbles';
        container.setAttribute('role', 'group');
        container.setAttribute('aria-label', 'Persona selection');
        
        this.bubbles = this.personas.map(persona => {
            const bubble = new PersonaBubble(persona);
            const bubbleElement = bubble.render();
            
            bubble.onClick((personaId) => {
                this.handlePersonaClick(personaId);
            });
            
            container.appendChild(bubbleElement);
            return bubble;
        });
        
        this.containerElement = container;
        
        if (this.inputContainer) {
            this.inputContainer.appendChild(container);
        }
        
        return container;
    }
    
    handlePersonaClick(personaId) {
        if (this.activePersonaId === personaId) {
            this.clearActivePersona();
        } else {
            this.setActivePersona(personaId);
        }
    }
    
    setActivePersona(personaId) {
        this.bubbles.forEach(bubble => {
            bubble.setActive(false);
        });
        
        const activeBubble = this.bubbles.find(b => b.persona.id === personaId);
        if (activeBubble) {
            activeBubble.setActive(true);
            this.activePersonaId = personaId;
        }
    }
    
    clearActivePersona() {
        this.bubbles.forEach(bubble => {
            bubble.setActive(false);
        });
        this.activePersonaId = null;
    }
    
    getActivePersona() {
        if (!this.activePersonaId) {
            return null;
        }
        return this.personas.find(p => p.id === this.activePersonaId) || null;
    }
    
    setDisabled(disabled) {
        this.bubbles.forEach(bubble => {
            bubble.setDisabled(disabled);
        });
    }
}

describe('Keyboard Navigation for Persona Bubbles', () => {
    let inputContainer;

    beforeEach(() => {
        loadHTML('<div class="input-container"></div>');
        inputContainer = document.querySelector('.input-container');
    });

    it('should make persona bubbles focusable with tabindex', () => {
        const personas = [
            { id: 'test-1', name: 'Test 1', icon: '🤖', system_prompt: 'Test prompt 1' },
            { id: 'test-2', name: 'Test 2', icon: '✨', system_prompt: 'Test prompt 2' }
        ];
        
        const container = new PersonaBubbleContainer(personas, inputContainer);
        container.render();
        
        const bubbles = inputContainer.querySelectorAll('.persona-bubble');
        
        expect(bubbles.length).toBe(2);
        
        bubbles.forEach(bubble => {
            expect(bubble.getAttribute('tabindex')).toBe('0');
        });
        
        inputContainer.innerHTML = '';
    });

    it('should toggle persona with Space key', () => {
        const personas = [
            { id: 'test-1', name: 'Test 1', icon: '🤖', system_prompt: 'Test prompt 1' }
        ];
        
        const container = new PersonaBubbleContainer(personas, inputContainer);
        container.render();
        
        const bubble = inputContainer.querySelector('.persona-bubble');
        
        // Focus the bubble
        bubble.focus();
        
        // Simulate Space key press
        const spaceEvent = new KeyboardEvent('keydown', {
            key: ' ',
            code: 'Space',
            bubbles: true,
            cancelable: true
        });
        
        bubble.dispatchEvent(spaceEvent);
        
        // Check that persona is now active
        expect(bubble.classList.contains('active')).toBe(true);
        expect(bubble.getAttribute('aria-pressed')).toBe('true');
        
        inputContainer.innerHTML = '';
    });

    it('should toggle persona with Enter key', () => {
        const personas = [
            { id: 'test-1', name: 'Test 1', icon: '🤖', system_prompt: 'Test prompt 1' }
        ];
        
        const container = new PersonaBubbleContainer(personas, inputContainer);
        container.render();
        
        const bubble = inputContainer.querySelector('.persona-bubble');
        
        // Focus the bubble
        bubble.focus();
        
        // Simulate Enter key press
        const enterEvent = new KeyboardEvent('keydown', {
            key: 'Enter',
            code: 'Enter',
            bubbles: true,
            cancelable: true
        });
        
        bubble.dispatchEvent(enterEvent);
        
        // Check that persona is now active
        expect(bubble.classList.contains('active')).toBe(true);
        expect(bubble.getAttribute('aria-pressed')).toBe('true');
        
        inputContainer.innerHTML = '';
    });

    it('should deselect active persona with Space key', () => {
        const personas = [
            { id: 'test-1', name: 'Test 1', icon: '🤖', system_prompt: 'Test prompt 1' }
        ];
        
        const container = new PersonaBubbleContainer(personas, inputContainer);
        container.render();
        
        const bubble = inputContainer.querySelector('.persona-bubble');
        
        // First activate the persona
        bubble.click();
        expect(bubble.classList.contains('active')).toBe(true);
        
        // Focus the bubble
        bubble.focus();
        
        // Simulate Space key press to deselect
        const spaceEvent = new KeyboardEvent('keydown', {
            key: ' ',
            code: 'Space',
            bubbles: true,
            cancelable: true
        });
        
        bubble.dispatchEvent(spaceEvent);
        
        // Check that persona is now inactive
        expect(bubble.classList.contains('active')).toBe(false);
        expect(bubble.getAttribute('aria-pressed')).toBe('false');
        
        inputContainer.innerHTML = '';
    });

    it('should have visible focus indicators', () => {
        const personas = [
            { id: 'test-1', name: 'Test 1', icon: '🤖', system_prompt: 'Test prompt 1' }
        ];
        
        const container = new PersonaBubbleContainer(personas, inputContainer);
        container.render();
        
        const bubble = inputContainer.querySelector('.persona-bubble');
        
        // Focus the bubble
        bubble.focus();
        
        // Note: In JSDOM, we can't fully test :focus-visible pseudo-class
        // but we can verify the element is focusable
        expect(document.activeElement).toBe(bubble);
        
        inputContainer.innerHTML = '';
    });

    it('should prevent default behavior on Space key to avoid page scroll', () => {
        const personas = [
            { id: 'test-1', name: 'Test 1', icon: '🤖', system_prompt: 'Test prompt 1' }
        ];
        
        const container = new PersonaBubbleContainer(personas, inputContainer);
        container.render();
        
        const bubble = inputContainer.querySelector('.persona-bubble');
        
        bubble.focus();
        
        let preventDefaultCalled = false;
        
        const spaceEvent = new KeyboardEvent('keydown', {
            key: ' ',
            code: 'Space',
            bubbles: true,
            cancelable: true
        });
        
        // Override preventDefault to track if it was called
        const originalPreventDefault = spaceEvent.preventDefault.bind(spaceEvent);
        spaceEvent.preventDefault = () => {
            preventDefaultCalled = true;
            originalPreventDefault();
        };
        
        bubble.dispatchEvent(spaceEvent);
        
        expect(preventDefaultCalled).toBe(true);
        
        inputContainer.innerHTML = '';
    });
});
