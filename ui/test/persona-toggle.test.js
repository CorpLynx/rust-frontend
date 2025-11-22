import { describe, it, expect, beforeEach } from 'vitest';
import fc from 'fast-check';
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
        button.dataset.personaId = this.persona.id;
        
        button.addEventListener('click', () => {
            if (this.clickCallback) {
                this.clickCallback(this.persona.id);
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

const personaArbitrary = fc.integer({ min: 0, max: 1000 }).chain(index => 
    fc.record({
        id: fc.constant(`persona-${index}`),
        name: fc.string({ minLength: 1, maxLength: 100 }),
        icon: fc.string({ minLength: 1, maxLength: 10 }),
        system_prompt: fc.string({ minLength: 1, maxLength: 500 }),
        description: fc.option(fc.string({ minLength: 0, maxLength: 200 }))
    })
);

const personaListArbitrary = fc.uniqueArray(personaArbitrary, {
    minLength: 1,
    maxLength: 10,
    selector: (persona) => persona.id
});

describe('Persona Toggle Behavior Property Tests', () => {
    let inputContainer;
    
    beforeEach(() => {
        loadHTML('<div class="input-container"></div>');
        inputContainer = document.querySelector('.input-container');
    });
    
    /**
     * Feature: persona-switcher, Property 4: Persona toggle behavior
     * Validates: Requirements 2.5
     * 
     * For any persona, clicking it when active should deselect it, 
     * and clicking it when inactive should activate it.
     */
    it('should activate persona when clicked while inactive', () => {
        fc.assert(
            fc.property(
                personaListArbitrary,
                fc.integer({ min: 0, max: 9 }),
                (personas, personaIndex) => {
                    if (personaIndex >= personas.length) return;
                    
                    const container = new PersonaBubbleContainer(personas, inputContainer);
                    container.render();
                    
                    const personaId = personas[personaIndex].id;
                    const bubble = container.bubbles[personaIndex];
                    
                    // Verify persona starts inactive
                    expect(bubble.isActive).toBe(false);
                    expect(container.activePersonaId).toBeNull();
                    
                    // Click inactive persona
                    container.handlePersonaClick(personaId);
                    
                    // Verify persona is now active
                    expect(bubble.isActive).toBe(true);
                    expect(container.activePersonaId).toBe(personaId);
                    expect(bubble.element.classList.contains('active')).toBe(true);
                    expect(bubble.element.getAttribute('aria-pressed')).toBe('true');
                    
                    inputContainer.innerHTML = '';
                }
            ),
            { numRuns: 100 }
        );
    });
    
    it('should deactivate persona when clicked while active', () => {
        fc.assert(
            fc.property(
                personaListArbitrary,
                fc.integer({ min: 0, max: 9 }),
                (personas, personaIndex) => {
                    if (personaIndex >= personas.length) return;
                    
                    const container = new PersonaBubbleContainer(personas, inputContainer);
                    container.render();
                    
                    const personaId = personas[personaIndex].id;
                    const bubble = container.bubbles[personaIndex];
                    
                    // First click to activate
                    container.handlePersonaClick(personaId);
                    expect(bubble.isActive).toBe(true);
                    expect(container.activePersonaId).toBe(personaId);
                    
                    // Second click to deactivate
                    container.handlePersonaClick(personaId);
                    
                    // Verify persona is now inactive
                    expect(bubble.isActive).toBe(false);
                    expect(container.activePersonaId).toBeNull();
                    expect(bubble.element.classList.contains('active')).toBe(false);
                    expect(bubble.element.getAttribute('aria-pressed')).toBe('false');
                    
                    inputContainer.innerHTML = '';
                }
            ),
            { numRuns: 100 }
        );
    });

    
    it('should toggle persona state correctly through multiple clicks', () => {
        fc.assert(
            fc.property(
                personaListArbitrary,
                fc.integer({ min: 0, max: 9 }),
                fc.integer({ min: 2, max: 10 }),
                (personas, personaIndex, numClicks) => {
                    if (personaIndex >= personas.length) return;
                    
                    const container = new PersonaBubbleContainer(personas, inputContainer);
                    container.render();
                    
                    const personaId = personas[personaIndex].id;
                    const bubble = container.bubbles[personaIndex];
                    
                    // Click multiple times and verify toggle behavior
                    for (let i = 0; i < numClicks; i++) {
                        const expectedActive = (i % 2 === 0); // Even clicks = active, odd = inactive
                        
                        container.handlePersonaClick(personaId);
                        
                        const actualActive = bubble.isActive;
                        const actualActiveId = container.activePersonaId;
                        
                        if (expectedActive) {
                            // After even number of clicks (1st, 3rd, 5th...), should be active
                            expect(actualActive).toBe(true);
                            expect(actualActiveId).toBe(personaId);
                            expect(bubble.element.classList.contains('active')).toBe(true);
                            expect(bubble.element.getAttribute('aria-pressed')).toBe('true');
                        } else {
                            // After odd number of clicks (2nd, 4th, 6th...), should be inactive
                            expect(actualActive).toBe(false);
                            expect(actualActiveId).toBeNull();
                            expect(bubble.element.classList.contains('active')).toBe(false);
                            expect(bubble.element.getAttribute('aria-pressed')).toBe('false');
                        }
                    }
                    
                    inputContainer.innerHTML = '';
                }
            ),
            { numRuns: 100 }
        );
    });
    
    it('should toggle any persona correctly regardless of position', () => {
        fc.assert(
            fc.property(
                personaListArbitrary,
                (personas) => {
                    const container = new PersonaBubbleContainer(personas, inputContainer);
                    container.render();
                    
                    // Test toggle behavior for each persona
                    for (let i = 0; i < personas.length; i++) {
                        const personaId = personas[i].id;
                        const bubble = container.bubbles[i];
                        
                        // Verify starts inactive
                        expect(bubble.isActive).toBe(false);
                        
                        // Click to activate
                        container.handlePersonaClick(personaId);
                        expect(bubble.isActive).toBe(true);
                        expect(container.activePersonaId).toBe(personaId);
                        
                        // Click to deactivate
                        container.handlePersonaClick(personaId);
                        expect(bubble.isActive).toBe(false);
                        expect(container.activePersonaId).toBeNull();
                    }
                    
                    inputContainer.innerHTML = '';
                }
            ),
            { numRuns: 100 }
        );
    });
    
    it('should maintain toggle behavior when switching between personas', () => {
        fc.assert(
            fc.property(
                fc.uniqueArray(personaArbitrary, { minLength: 2, maxLength: 10, selector: (p) => p.id }),
                fc.integer({ min: 0, max: 9 }),
                fc.integer({ min: 0, max: 9 }),
                (personas, firstIndex, secondIndex) => {
                    if (firstIndex >= personas.length || secondIndex >= personas.length) return;
                    if (firstIndex === secondIndex) return;
                    
                    const container = new PersonaBubbleContainer(personas, inputContainer);
                    container.render();
                    
                    const firstPersonaId = personas[firstIndex].id;
                    const secondPersonaId = personas[secondIndex].id;
                    const firstBubble = container.bubbles[firstIndex];
                    const secondBubble = container.bubbles[secondIndex];
                    
                    // Activate first persona
                    container.handlePersonaClick(firstPersonaId);
                    expect(firstBubble.isActive).toBe(true);
                    expect(container.activePersonaId).toBe(firstPersonaId);
                    
                    // Activate second persona (should deactivate first)
                    container.handlePersonaClick(secondPersonaId);
                    expect(firstBubble.isActive).toBe(false);
                    expect(secondBubble.isActive).toBe(true);
                    expect(container.activePersonaId).toBe(secondPersonaId);
                    
                    // Click second persona again (should deactivate it)
                    container.handlePersonaClick(secondPersonaId);
                    expect(secondBubble.isActive).toBe(false);
                    expect(container.activePersonaId).toBeNull();
                    
                    // Click first persona again (should activate it)
                    container.handlePersonaClick(firstPersonaId);
                    expect(firstBubble.isActive).toBe(true);
                    expect(container.activePersonaId).toBe(firstPersonaId);
                    
                    inputContainer.innerHTML = '';
                }
            ),
            { numRuns: 100 }
        );
    });
    
    it('should update visual state correctly during toggle', () => {
        fc.assert(
            fc.property(
                personaListArbitrary,
                fc.integer({ min: 0, max: 9 }),
                (personas, personaIndex) => {
                    if (personaIndex >= personas.length) return;
                    
                    const container = new PersonaBubbleContainer(personas, inputContainer);
                    container.render();
                    
                    const personaId = personas[personaIndex].id;
                    const bubble = container.bubbles[personaIndex];
                    
                    // Initial state - inactive
                    expect(bubble.element.classList.contains('active')).toBe(false);
                    expect(bubble.element.getAttribute('aria-pressed')).toBe('false');
                    
                    // Click to activate
                    container.handlePersonaClick(personaId);
                    expect(bubble.element.classList.contains('active')).toBe(true);
                    expect(bubble.element.getAttribute('aria-pressed')).toBe('true');
                    
                    // Click to deactivate
                    container.handlePersonaClick(personaId);
                    expect(bubble.element.classList.contains('active')).toBe(false);
                    expect(bubble.element.getAttribute('aria-pressed')).toBe('false');
                    
                    // Verify all other bubbles remain inactive
                    container.bubbles.forEach((b, idx) => {
                        if (idx !== personaIndex) {
                            expect(b.element.classList.contains('active')).toBe(false);
                            expect(b.element.getAttribute('aria-pressed')).toBe('false');
                        }
                    });
                    
                    inputContainer.innerHTML = '';
                }
            ),
            { numRuns: 100 }
        );
    });
});
