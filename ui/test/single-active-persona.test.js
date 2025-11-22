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
    minLength: 2,
    maxLength: 10,
    selector: (persona) => persona.id
});

const personaSelectionSequenceArbitrary = (personas) => {
    return fc.array(
        fc.integer({ min: 0, max: personas.length - 1 }),
        { minLength: 1, maxLength: 20 }
    );
};

describe('Single Active Persona Invariant Property Tests', () => {
    let inputContainer;
    
    beforeEach(() => {
        loadHTML('<div class="input-container"></div>');
        inputContainer = document.querySelector('.input-container');
    });

    
    /**
     * Feature: persona-switcher, Property 3: Single active persona invariant
     * Validates: Requirements 2.1, 2.2, 2.3
     * 
     * For any sequence of persona selections, at most one persona should be active 
     * at any given time, with visual highlighting applied only to the active persona.
     */
    it('should maintain at most one active persona across any selection sequence', () => {
        fc.assert(
            fc.property(personaListArbitrary, (personas) => {
                const container = new PersonaBubbleContainer(personas, inputContainer);
                container.render();
                
                const selectionSequence = fc.sample(
                    personaSelectionSequenceArbitrary(personas),
                    1
                )[0];
                
                for (const personaIndex of selectionSequence) {
                    const personaId = personas[personaIndex].id;
                    container.handlePersonaClick(personaId);
                    
                    const activeBubbles = container.bubbles.filter(b => b.isActive);
                    expect(activeBubbles.length).toBeLessThanOrEqual(1);
                    
                    if (activeBubbles.length === 1) {
                        expect(activeBubbles[0].persona.id).toBe(container.activePersonaId);
                    } else {
                        expect(container.activePersonaId).toBeNull();
                    }
                    
                    container.bubbles.forEach(bubble => {
                        const hasActiveClass = bubble.element.classList.contains('active');
                        const ariaPressed = bubble.element.getAttribute('aria-pressed');
                        
                        if (bubble.isActive) {
                            expect(hasActiveClass).toBe(true);
                            expect(ariaPressed).toBe('true');
                        } else {
                            expect(hasActiveClass).toBe(false);
                            expect(ariaPressed).toBe('false');
                        }
                    });
                    
                    const activePersona = container.getActivePersona();
                    if (container.activePersonaId) {
                        expect(activePersona).not.toBeNull();
                        expect(activePersona.id).toBe(container.activePersonaId);
                    } else {
                        expect(activePersona).toBeNull();
                    }
                }
                
                inputContainer.innerHTML = '';
            }),
            { numRuns: 100 }
        );
    });
    
    it('should ensure only one persona has active class after any selection', () => {
        fc.assert(
            fc.property(personaListArbitrary, (personas) => {
                const container = new PersonaBubbleContainer(personas, inputContainer);
                container.render();
                
                const selectionSequence = fc.sample(
                    personaSelectionSequenceArbitrary(personas),
                    1
                )[0];
                
                for (const personaIndex of selectionSequence) {
                    const personaId = personas[personaIndex].id;
                    container.handlePersonaClick(personaId);
                    
                    const bubblesWithActiveClass = container.bubbles.filter(
                        b => b.element.classList.contains('active')
                    );
                    expect(bubblesWithActiveClass.length).toBeLessThanOrEqual(1);
                    
                    const bubblesWithAriaPressed = container.bubbles.filter(
                        b => b.element.getAttribute('aria-pressed') === 'true'
                    );
                    expect(bubblesWithAriaPressed.length).toBeLessThanOrEqual(1);
                    expect(bubblesWithActiveClass.length).toBe(bubblesWithAriaPressed.length);
                }
                
                inputContainer.innerHTML = '';
            }),
            { numRuns: 100 }
        );
    });

    
    it('should maintain single active persona when clicking different personas', () => {
        fc.assert(
            fc.property(
                personaListArbitrary,
                fc.array(fc.integer({ min: 0, max: 9 }), { minLength: 5, maxLength: 15 }),
                (personas, selectionIndices) => {
                    const validIndices = selectionIndices.filter(i => i < personas.length);
                    if (validIndices.length === 0) return;
                    
                    const container = new PersonaBubbleContainer(personas, inputContainer);
                    container.render();
                    
                    for (const personaIndex of validIndices) {
                        const personaId = personas[personaIndex].id;
                        container.handlePersonaClick(personaId);
                        
                        const activeCount = container.bubbles.filter(b => b.isActive).length;
                        expect(activeCount).toBeGreaterThanOrEqual(0);
                        expect(activeCount).toBeLessThanOrEqual(1);
                        
                        if (activeCount === 1) {
                            const activeBubble = container.bubbles.find(b => b.isActive);
                            expect(activeBubble.persona.id).toBe(container.activePersonaId);
                        } else {
                            expect(container.activePersonaId).toBeNull();
                        }
                    }
                    
                    inputContainer.innerHTML = '';
                }
            ),
            { numRuns: 100 }
        );
    });
    
    it('should deactivate previous persona when activating a new one', () => {
        fc.assert(
            fc.property(
                personaListArbitrary,
                fc.integer({ min: 0, max: 9 }),
                fc.integer({ min: 0, max: 9 }),
                (personas, firstIndex, secondIndex) => {
                    if (firstIndex >= personas.length || secondIndex >= personas.length) return;
                    if (firstIndex === secondIndex) return;
                    
                    const container = new PersonaBubbleContainer(personas, inputContainer);
                    container.render();
                    
                    const firstPersonaId = personas[firstIndex].id;
                    container.handlePersonaClick(firstPersonaId);
                    
                    expect(container.activePersonaId).toBe(firstPersonaId);
                    expect(container.bubbles[firstIndex].isActive).toBe(true);
                    
                    const secondPersonaId = personas[secondIndex].id;
                    container.handlePersonaClick(secondPersonaId);
                    
                    expect(container.bubbles[firstIndex].isActive).toBe(false);
                    expect(container.bubbles[firstIndex].element.classList.contains('active')).toBe(false);
                    expect(container.bubbles[firstIndex].element.getAttribute('aria-pressed')).toBe('false');
                    
                    expect(container.activePersonaId).toBe(secondPersonaId);
                    expect(container.bubbles[secondIndex].isActive).toBe(true);
                    expect(container.bubbles[secondIndex].element.classList.contains('active')).toBe(true);
                    expect(container.bubbles[secondIndex].element.getAttribute('aria-pressed')).toBe('true');
                    
                    const activeCount = container.bubbles.filter(b => b.isActive).length;
                    expect(activeCount).toBe(1);
                    
                    inputContainer.innerHTML = '';
                }
            ),
            { numRuns: 100 }
        );
    });

    
    it('should handle toggling (clicking active persona deselects it)', () => {
        fc.assert(
            fc.property(
                personaListArbitrary,
                fc.integer({ min: 0, max: 9 }),
                (personas, personaIndex) => {
                    if (personaIndex >= personas.length) return;
                    
                    const container = new PersonaBubbleContainer(personas, inputContainer);
                    container.render();
                    
                    const personaId = personas[personaIndex].id;
                    
                    container.handlePersonaClick(personaId);
                    
                    expect(container.activePersonaId).toBe(personaId);
                    expect(container.bubbles[personaIndex].isActive).toBe(true);
                    
                    container.handlePersonaClick(personaId);
                    
                    expect(container.activePersonaId).toBeNull();
                    expect(container.bubbles[personaIndex].isActive).toBe(false);
                    
                    const activeCount = container.bubbles.filter(b => b.isActive).length;
                    expect(activeCount).toBe(0);
                    
                    container.bubbles.forEach(bubble => {
                        expect(bubble.element.classList.contains('active')).toBe(false);
                        expect(bubble.element.getAttribute('aria-pressed')).toBe('false');
                    });
                    
                    inputContainer.innerHTML = '';
                }
            ),
            { numRuns: 100 }
        );
    });
    
    it('should maintain invariant when using setActivePersona directly', () => {
        fc.assert(
            fc.property(
                personaListArbitrary,
                fc.array(fc.integer({ min: 0, max: 9 }), { minLength: 1, maxLength: 10 }),
                (personas, selectionIndices) => {
                    const validIndices = selectionIndices.filter(i => i < personas.length);
                    if (validIndices.length === 0) return;
                    
                    const container = new PersonaBubbleContainer(personas, inputContainer);
                    container.render();
                    
                    for (const personaIndex of validIndices) {
                        const personaId = personas[personaIndex].id;
                        container.setActivePersona(personaId);
                        
                        const activeCount = container.bubbles.filter(b => b.isActive).length;
                        expect(activeCount).toBe(1);
                        
                        expect(container.activePersonaId).toBe(personaId);
                        expect(container.bubbles[personaIndex].isActive).toBe(true);
                        
                        container.bubbles.forEach((bubble, index) => {
                            if (index !== personaIndex) {
                                expect(bubble.isActive).toBe(false);
                            }
                        });
                    }
                    
                    inputContainer.innerHTML = '';
                }
            ),
            { numRuns: 100 }
        );
    });
});
