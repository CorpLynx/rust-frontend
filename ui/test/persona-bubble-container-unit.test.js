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

// PersonaBubbleContainer component for testing
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
            localStorage.setItem('activePersonaId', personaId);
        }
    }
    
    clearActivePersona() {
        this.bubbles.forEach(bubble => {
            bubble.setActive(false);
        });
        this.activePersonaId = null;
        localStorage.removeItem('activePersonaId');
    }
    
    restoreActivePersona() {
        const savedPersonaId = localStorage.getItem('activePersonaId');
        if (savedPersonaId) {
            const personaExists = this.personas.some(p => p.id === savedPersonaId);
            if (personaExists) {
                this.setActivePersona(savedPersonaId);
            } else {
                localStorage.removeItem('activePersonaId');
            }
        }
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

describe('PersonaBubbleContainer Unit Tests', () => {
    let inputContainer;
    
    beforeEach(() => {
        loadHTML('<div class="input-container"></div>');
        inputContainer = document.querySelector('.input-container');
        // Clear localStorage manually since jsdom doesn't support clear()
        localStorage.removeItem('activePersonaId');
    });
    
    describe('Container rendering', () => {
        it('should render container with correct class and attributes', () => {
            const personas = [
                { id: '1', name: 'Test 1', icon: '🤖', system_prompt: 'test' },
                { id: '2', name: 'Test 2', icon: '✍️', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            const containerElement = inputContainer.querySelector('.persona-bubbles');
            expect(containerElement).not.toBeNull();
            expect(containerElement.getAttribute('role')).toBe('group');
            expect(containerElement.getAttribute('aria-label')).toBe('Persona selection');
        });
        
        it('should render all persona bubbles', () => {
            const personas = [
                { id: '1', name: 'Test 1', icon: '🤖', system_prompt: 'test' },
                { id: '2', name: 'Test 2', icon: '✍️', system_prompt: 'test' },
                { id: '3', name: 'Test 3', icon: '💻', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            expect(container.bubbles.length).toBe(3);
            
            const bubbleElements = inputContainer.querySelectorAll('.persona-bubble');
            expect(bubbleElements.length).toBe(3);
        });
        
        it('should render empty container with no personas', () => {
            const personas = [];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            expect(container.bubbles.length).toBe(0);
            
            const containerElement = inputContainer.querySelector('.persona-bubbles');
            expect(containerElement).not.toBeNull();
            expect(containerElement.children.length).toBe(0);
        });
        
        it('should append container to input container', () => {
            const personas = [
                { id: '1', name: 'Test', icon: '🤖', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            expect(inputContainer.children.length).toBe(1);
            expect(inputContainer.firstChild.className).toBe('persona-bubbles');
        });
        
        it('should work without input container', () => {
            const personas = [
                { id: '1', name: 'Test', icon: '🤖', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, null);
            const element = container.render();
            
            expect(element).not.toBeNull();
            expect(element.className).toBe('persona-bubbles');
            expect(container.bubbles.length).toBe(1);
        });
    });
    
    describe('Active persona management', () => {
        it('should set active persona correctly', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' },
                { id: 'p2', name: 'Persona 2', icon: '✍️', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            container.setActivePersona('p1');
            
            expect(container.activePersonaId).toBe('p1');
            expect(container.bubbles[0].isActive).toBe(true);
            expect(container.bubbles[1].isActive).toBe(false);
        });
        
        it('should clear previous active when setting new active', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' },
                { id: 'p2', name: 'Persona 2', icon: '✍️', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            container.setActivePersona('p1');
            expect(container.bubbles[0].isActive).toBe(true);
            
            container.setActivePersona('p2');
            expect(container.bubbles[0].isActive).toBe(false);
            expect(container.bubbles[1].isActive).toBe(true);
            expect(container.activePersonaId).toBe('p2');
        });
        
        it('should clear all active personas', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' },
                { id: 'p2', name: 'Persona 2', icon: '✍️', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            container.setActivePersona('p1');
            container.clearActivePersona();
            
            expect(container.activePersonaId).toBeNull();
            expect(container.bubbles[0].isActive).toBe(false);
            expect(container.bubbles[1].isActive).toBe(false);
        });
        
        it('should return active persona object', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'You are helpful' },
                { id: 'p2', name: 'Persona 2', icon: '✍️', system_prompt: 'You are creative' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            container.setActivePersona('p2');
            const activePersona = container.getActivePersona();
            
            expect(activePersona).not.toBeNull();
            expect(activePersona.id).toBe('p2');
            expect(activePersona.name).toBe('Persona 2');
            expect(activePersona.system_prompt).toBe('You are creative');
        });
        
        it('should return null when no active persona', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            const activePersona = container.getActivePersona();
            expect(activePersona).toBeNull();
        });
        
        it('should handle invalid persona ID gracefully', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            container.setActivePersona('invalid-id');
            
            expect(container.activePersonaId).toBeNull();
            expect(container.bubbles[0].isActive).toBe(false);
        });
    });
    
    describe('Bubble positioning', () => {
        it('should position bubbles as children of container', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' },
                { id: 'p2', name: 'Persona 2', icon: '✍️', system_prompt: 'test' },
                { id: 'p3', name: 'Persona 3', icon: '💻', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            const containerElement = inputContainer.querySelector('.persona-bubbles');
            const bubbleElements = containerElement.querySelectorAll('.persona-bubble');
            
            expect(bubbleElements.length).toBe(3);
            
            // Verify each bubble is a direct child
            bubbleElements.forEach((bubble, index) => {
                expect(bubble.parentElement).toBe(containerElement);
                expect(bubble.dataset.personaId).toBe(personas[index].id);
            });
        });
        
        it('should maintain bubble order matching persona array', () => {
            const personas = [
                { id: 'first', name: 'First', icon: '1️⃣', system_prompt: 'test' },
                { id: 'second', name: 'Second', icon: '2️⃣', system_prompt: 'test' },
                { id: 'third', name: 'Third', icon: '3️⃣', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            const bubbleElements = inputContainer.querySelectorAll('.persona-bubble');
            
            expect(bubbleElements[0].dataset.personaId).toBe('first');
            expect(bubbleElements[1].dataset.personaId).toBe('second');
            expect(bubbleElements[2].dataset.personaId).toBe('third');
        });
    });
    
    describe('Click handling', () => {
        it('should activate persona on click', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            container.handlePersonaClick('p1');
            
            expect(container.activePersonaId).toBe('p1');
            expect(container.bubbles[0].isActive).toBe(true);
        });
        
        it('should deactivate persona on second click', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            container.handlePersonaClick('p1');
            expect(container.activePersonaId).toBe('p1');
            
            container.handlePersonaClick('p1');
            expect(container.activePersonaId).toBeNull();
            expect(container.bubbles[0].isActive).toBe(false);
        });
        
        it('should switch between personas on click', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' },
                { id: 'p2', name: 'Persona 2', icon: '✍️', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            container.handlePersonaClick('p1');
            expect(container.activePersonaId).toBe('p1');
            
            container.handlePersonaClick('p2');
            expect(container.activePersonaId).toBe('p2');
            expect(container.bubbles[0].isActive).toBe(false);
            expect(container.bubbles[1].isActive).toBe(true);
        });
    });
    
    describe('Disabled state', () => {
        it('should disable all bubbles', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' },
                { id: 'p2', name: 'Persona 2', icon: '✍️', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            container.setDisabled(true);
            
            const bubbleElements = inputContainer.querySelectorAll('.persona-bubble');
            bubbleElements.forEach(bubble => {
                expect(bubble.disabled).toBe(true);
            });
        });
        
        it('should enable all bubbles', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' },
                { id: 'p2', name: 'Persona 2', icon: '✍️', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            container.setDisabled(true);
            container.setDisabled(false);
            
            const bubbleElements = inputContainer.querySelectorAll('.persona-bubble');
            bubbleElements.forEach(bubble => {
                expect(bubble.disabled).toBe(false);
            });
        });
    });
    
    describe('LocalStorage persistence', () => {
        it('should save active persona to localStorage', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            container.setActivePersona('p1');
            
            expect(localStorage.getItem('activePersonaId')).toBe('p1');
        });
        
        it('should remove from localStorage when cleared', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            
            container.setActivePersona('p1');
            container.clearActivePersona();
            
            expect(localStorage.getItem('activePersonaId')).toBeNull();
        });
        
        it('should restore active persona from localStorage', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' },
                { id: 'p2', name: 'Persona 2', icon: '✍️', system_prompt: 'test' }
            ];
            
            localStorage.setItem('activePersonaId', 'p2');
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            container.restoreActivePersona();
            
            expect(container.activePersonaId).toBe('p2');
            expect(container.bubbles[1].isActive).toBe(true);
        });
        
        it('should handle invalid persona ID in localStorage', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' }
            ];
            
            localStorage.setItem('activePersonaId', 'invalid-id');
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            container.restoreActivePersona();
            
            expect(container.activePersonaId).toBeNull();
            expect(localStorage.getItem('activePersonaId')).toBeNull();
        });
        
        it('should not restore if localStorage is empty', () => {
            const personas = [
                { id: 'p1', name: 'Persona 1', icon: '🤖', system_prompt: 'test' }
            ];
            
            const container = new PersonaBubbleContainer(personas, inputContainer);
            container.render();
            container.restoreActivePersona();
            
            expect(container.activePersonaId).toBeNull();
        });
    });
});
