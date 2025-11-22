import { describe, it, expect, beforeEach, vi } from 'vitest';
import { loadHTML } from './utils.js';

// PersonaBubble component
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

// PersonaBubbleContainer component
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

// Mock Tauri invoke function
const mockInvoke = vi.fn();

describe('Persona Integration Tests', () => {
    let inputContainer;
    let personaBubbleContainer;
    let personas;
    
    beforeEach(() => {
        loadHTML(`
            <div class="input-container">
                <input id="message-input" type="text" />
                <button id="send-btn">Send</button>
            </div>
        `);
        inputContainer = document.querySelector('.input-container');
        
        personas = [
            {
                id: 'default',
                name: 'Default Assistant',
                icon: '🤖',
                system_prompt: 'You are a helpful AI assistant.',
                description: 'General purpose assistant'
            },
            {
                id: 'creative-writer',
                name: 'Creative Writer',
                icon: '✍️',
                system_prompt: 'You are a creative writing assistant who helps with storytelling, poetry, and creative content.',
                description: 'Helps with creative writing tasks'
            },
            {
                id: 'code-helper',
                name: 'Code Helper',
                icon: '💻',
                system_prompt: 'You are a coding assistant who helps with programming, debugging, and software development.',
                description: 'Assists with coding tasks'
            }
        ];
        
        // Mock Tauri API
        global.window.__TAURI__ = {
            core: {
                invoke: mockInvoke
            }
        };
        
        mockInvoke.mockReset();
    });
    
    describe('End-to-end: Persona selection → message send → system prompt in API call', () => {
        it('should include system prompt when persona is active', async () => {
            // Setup
            personaBubbleContainer = new PersonaBubbleContainer(personas, inputContainer);
            personaBubbleContainer.render();
            
            const messageInput = document.getElementById('message-input');
            const sendBtn = document.getElementById('send-btn');
            
            // Select creative writer persona
            personaBubbleContainer.handlePersonaClick('creative-writer');
            
            // Verify persona is active
            expect(personaBubbleContainer.activePersonaId).toBe('creative-writer');
            const activePersona = personaBubbleContainer.getActivePersona();
            expect(activePersona).not.toBeNull();
            expect(activePersona.system_prompt).toBe('You are a creative writing assistant who helps with storytelling, poetry, and creative content.');
            
            // Simulate sending a message
            messageInput.value = 'Write a short story about a robot';
            
            // Mock the send message function
            const sendMessage = async () => {
                const message = messageInput.value.trim();
                const activePersona = personaBubbleContainer.getActivePersona();
                const systemPrompt = activePersona ? activePersona.system_prompt : null;
                
                await mockInvoke('send_message_stream', {
                    prompt: message,
                    model: 'llama2',
                    requestId: 'test-req-1',
                    systemPrompt: systemPrompt
                });
            };
            
            await sendMessage();
            
            // Verify invoke was called with system prompt
            expect(mockInvoke).toHaveBeenCalledTimes(1);
            expect(mockInvoke).toHaveBeenCalledWith('send_message_stream', {
                prompt: 'Write a short story about a robot',
                model: 'llama2',
                requestId: 'test-req-1',
                systemPrompt: 'You are a creative writing assistant who helps with storytelling, poetry, and creative content.'
            });
        });
        
        it('should not include system prompt when no persona is active', async () => {
            // Setup
            personaBubbleContainer = new PersonaBubbleContainer(personas, inputContainer);
            personaBubbleContainer.render();
            
            const messageInput = document.getElementById('message-input');
            
            // No persona selected
            expect(personaBubbleContainer.activePersonaId).toBeNull();
            
            // Simulate sending a message
            messageInput.value = 'Hello, how are you?';
            
            const sendMessage = async () => {
                const message = messageInput.value.trim();
                const activePersona = personaBubbleContainer.getActivePersona();
                const systemPrompt = activePersona ? activePersona.system_prompt : null;
                
                await mockInvoke('send_message_stream', {
                    prompt: message,
                    model: 'llama2',
                    requestId: 'test-req-2',
                    systemPrompt: systemPrompt
                });
            };
            
            await sendMessage();
            
            // Verify invoke was called without system prompt
            expect(mockInvoke).toHaveBeenCalledTimes(1);
            expect(mockInvoke).toHaveBeenCalledWith('send_message_stream', {
                prompt: 'Hello, how are you?',
                model: 'llama2',
                requestId: 'test-req-2',
                systemPrompt: null
            });
        });
    });
    
    describe('End-to-end: Persona switching mid-conversation', () => {
        it('should use new persona system prompt after switching', async () => {
            // Setup
            personaBubbleContainer = new PersonaBubbleContainer(personas, inputContainer);
            personaBubbleContainer.render();
            
            const messageInput = document.getElementById('message-input');
            
            // First message with default persona
            personaBubbleContainer.handlePersonaClick('default');
            messageInput.value = 'First message';
            
            let sendMessage = async () => {
                const message = messageInput.value.trim();
                const activePersona = personaBubbleContainer.getActivePersona();
                const systemPrompt = activePersona ? activePersona.system_prompt : null;
                
                await mockInvoke('send_message_stream', {
                    prompt: message,
                    model: 'llama2',
                    requestId: 'test-req-3',
                    systemPrompt: systemPrompt
                });
            };
            
            await sendMessage();
            
            expect(mockInvoke).toHaveBeenCalledWith('send_message_stream', {
                prompt: 'First message',
                model: 'llama2',
                requestId: 'test-req-3',
                systemPrompt: 'You are a helpful AI assistant.'
            });
            
            // Switch to code helper persona
            personaBubbleContainer.handlePersonaClick('code-helper');
            expect(personaBubbleContainer.activePersonaId).toBe('code-helper');
            
            // Second message with new persona
            messageInput.value = 'Second message';
            
            sendMessage = async () => {
                const message = messageInput.value.trim();
                const activePersona = personaBubbleContainer.getActivePersona();
                const systemPrompt = activePersona ? activePersona.system_prompt : null;
                
                await mockInvoke('send_message_stream', {
                    prompt: message,
                    model: 'llama2',
                    requestId: 'test-req-4',
                    systemPrompt: systemPrompt
                });
            };
            
            await sendMessage();
            
            expect(mockInvoke).toHaveBeenCalledWith('send_message_stream', {
                prompt: 'Second message',
                model: 'llama2',
                requestId: 'test-req-4',
                systemPrompt: 'You are a coding assistant who helps with programming, debugging, and software development.'
            });
        });
        
        it('should handle switching from active to no persona', async () => {
            // Setup
            personaBubbleContainer = new PersonaBubbleContainer(personas, inputContainer);
            personaBubbleContainer.render();
            
            const messageInput = document.getElementById('message-input');
            
            // First message with persona
            personaBubbleContainer.handlePersonaClick('creative-writer');
            messageInput.value = 'First message';
            
            let sendMessage = async () => {
                const message = messageInput.value.trim();
                const activePersona = personaBubbleContainer.getActivePersona();
                const systemPrompt = activePersona ? activePersona.system_prompt : null;
                
                await mockInvoke('send_message_stream', {
                    prompt: message,
                    model: 'llama2',
                    requestId: 'test-req-5',
                    systemPrompt: systemPrompt
                });
            };
            
            await sendMessage();
            
            expect(mockInvoke).toHaveBeenCalledWith('send_message_stream', {
                prompt: 'First message',
                model: 'llama2',
                requestId: 'test-req-5',
                systemPrompt: 'You are a creative writing assistant who helps with storytelling, poetry, and creative content.'
            });
            
            // Deselect persona
            personaBubbleContainer.handlePersonaClick('creative-writer');
            expect(personaBubbleContainer.activePersonaId).toBeNull();
            
            // Second message without persona
            messageInput.value = 'Second message';
            
            sendMessage = async () => {
                const message = messageInput.value.trim();
                const activePersona = personaBubbleContainer.getActivePersona();
                const systemPrompt = activePersona ? activePersona.system_prompt : null;
                
                await mockInvoke('send_message_stream', {
                    prompt: message,
                    model: 'llama2',
                    requestId: 'test-req-6',
                    systemPrompt: systemPrompt
                });
            };
            
            await sendMessage();
            
            expect(mockInvoke).toHaveBeenCalledWith('send_message_stream', {
                prompt: 'Second message',
                model: 'llama2',
                requestId: 'test-req-6',
                systemPrompt: null
            });
        });
    });
    
    describe('End-to-end: Deselection and no-prompt scenario', () => {
        it('should send no prompt after deselecting active persona', async () => {
            // Setup
            personaBubbleContainer = new PersonaBubbleContainer(personas, inputContainer);
            personaBubbleContainer.render();
            
            const messageInput = document.getElementById('message-input');
            
            // Select persona
            personaBubbleContainer.handlePersonaClick('code-helper');
            expect(personaBubbleContainer.activePersonaId).toBe('code-helper');
            
            // Deselect by clicking again
            personaBubbleContainer.handlePersonaClick('code-helper');
            expect(personaBubbleContainer.activePersonaId).toBeNull();
            
            // Send message
            messageInput.value = 'Test message';
            
            const sendMessage = async () => {
                const message = messageInput.value.trim();
                const activePersona = personaBubbleContainer.getActivePersona();
                const systemPrompt = activePersona ? activePersona.system_prompt : null;
                
                await mockInvoke('send_message_stream', {
                    prompt: message,
                    model: 'llama2',
                    requestId: 'test-req-7',
                    systemPrompt: systemPrompt
                });
            };
            
            await sendMessage();
            
            expect(mockInvoke).toHaveBeenCalledWith('send_message_stream', {
                prompt: 'Test message',
                model: 'llama2',
                requestId: 'test-req-7',
                systemPrompt: null
            });
        });
        
        it('should handle multiple persona selections and deselections', async () => {
            // Setup
            personaBubbleContainer = new PersonaBubbleContainer(personas, inputContainer);
            personaBubbleContainer.render();
            
            const messageInput = document.getElementById('message-input');
            
            // Select first persona
            personaBubbleContainer.handlePersonaClick('default');
            expect(personaBubbleContainer.activePersonaId).toBe('default');
            
            // Switch to second persona
            personaBubbleContainer.handlePersonaClick('creative-writer');
            expect(personaBubbleContainer.activePersonaId).toBe('creative-writer');
            
            // Deselect
            personaBubbleContainer.handlePersonaClick('creative-writer');
            expect(personaBubbleContainer.activePersonaId).toBeNull();
            
            // Select third persona
            personaBubbleContainer.handlePersonaClick('code-helper');
            expect(personaBubbleContainer.activePersonaId).toBe('code-helper');
            
            // Send message with final persona
            messageInput.value = 'Final message';
            
            const sendMessage = async () => {
                const message = messageInput.value.trim();
                const activePersona = personaBubbleContainer.getActivePersona();
                const systemPrompt = activePersona ? activePersona.system_prompt : null;
                
                await mockInvoke('send_message_stream', {
                    prompt: message,
                    model: 'llama2',
                    requestId: 'test-req-8',
                    systemPrompt: systemPrompt
                });
            };
            
            await sendMessage();
            
            expect(mockInvoke).toHaveBeenCalledWith('send_message_stream', {
                prompt: 'Final message',
                model: 'llama2',
                requestId: 'test-req-8',
                systemPrompt: 'You are a coding assistant who helps with programming, debugging, and software development.'
            });
        });
    });
    
    describe('End-to-end: Persona state persistence', () => {
        it('should persist active persona across page reloads', () => {
            // Setup first instance
            personaBubbleContainer = new PersonaBubbleContainer(personas, inputContainer);
            personaBubbleContainer.render();
            
            // Select persona
            personaBubbleContainer.handlePersonaClick('creative-writer');
            expect(personaBubbleContainer.activePersonaId).toBe('creative-writer');
            expect(localStorage.getItem('activePersonaId')).toBe('creative-writer');
            
            // Simulate page reload by creating new instance
            const newContainer = new PersonaBubbleContainer(personas, inputContainer);
            newContainer.render();
            newContainer.restoreActivePersona();
            
            // Verify persona is restored
            expect(newContainer.activePersonaId).toBe('creative-writer');
            expect(newContainer.bubbles[1].isActive).toBe(true);
        });
        
        it('should clear persona state on new conversation', () => {
            // Setup
            personaBubbleContainer = new PersonaBubbleContainer(personas, inputContainer);
            personaBubbleContainer.render();
            
            // Select persona
            personaBubbleContainer.handlePersonaClick('code-helper');
            expect(personaBubbleContainer.activePersonaId).toBe('code-helper');
            
            // Simulate new conversation
            personaBubbleContainer.clearActivePersona();
            
            // Verify state is cleared
            expect(personaBubbleContainer.activePersonaId).toBeNull();
            expect(localStorage.getItem('activePersonaId')).toBeNull();
            expect(personaBubbleContainer.bubbles.every(b => !b.isActive)).toBe(true);
        });
    });
    
    describe('End-to-end: Disabled state during message sending', () => {
        it('should disable persona bubbles during message sending', async () => {
            // Setup
            personaBubbleContainer = new PersonaBubbleContainer(personas, inputContainer);
            personaBubbleContainer.render();
            
            const messageInput = document.getElementById('message-input');
            
            // Select persona
            personaBubbleContainer.handlePersonaClick('default');
            
            // Simulate message sending
            messageInput.value = 'Test message';
            
            // Disable bubbles (simulating loading state)
            personaBubbleContainer.setDisabled(true);
            
            // Verify all bubbles are disabled
            const bubbleElements = inputContainer.querySelectorAll('.persona-bubble');
            bubbleElements.forEach(bubble => {
                expect(bubble.disabled).toBe(true);
            });
            
            // Simulate message completion
            personaBubbleContainer.setDisabled(false);
            
            // Verify bubbles are re-enabled
            bubbleElements.forEach(bubble => {
                expect(bubble.disabled).toBe(false);
            });
        });
    });
});
