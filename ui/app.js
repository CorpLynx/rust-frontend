// Check if Tauri API is available
if (!window.__TAURI__) {
    console.error('Tauri API not available! Make sure you are running this in a Tauri app.');
}

const { invoke } = window.__TAURI__?.core || { invoke: () => Promise.reject('Tauri not available') };

let currentModel = null;
let isLoading = false;
let sidebarOpen = false;
let isAnimating = false;
let currentConversationId = null;
let conversations = [];

// PersonaBubble Component
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

// PersonaBubbleContainer Component
class PersonaBubbleContainer {
    constructor(personas, inputContainer) {
        this.personas = personas;
        this.inputContainer = inputContainer;
        this.bubbles = [];
        this.activePersonaId = null;
        this.containerElement = null;
    }
    
    render() {
        // Create container element
        const container = document.createElement('div');
        container.className = 'persona-bubbles';
        container.setAttribute('role', 'group');
        container.setAttribute('aria-label', 'Persona selection');
        
        // Create persona bubbles
        this.bubbles = this.personas.map(persona => {
            const bubble = new PersonaBubble(persona);
            const bubbleElement = bubble.render();
            
            // Set up click handler for persona selection
            bubble.onClick((personaId) => {
                this.handlePersonaClick(personaId);
            });
            
            container.appendChild(bubbleElement);
            return bubble;
        });
        
        this.containerElement = container;
        
        // Append to input container
        if (this.inputContainer) {
            this.inputContainer.appendChild(container);
        }
        
        return container;
    }
    
    handlePersonaClick(personaId) {
        if (this.activePersonaId === personaId) {
            // Clicking active persona deselects it
            this.clearActivePersona();
        } else {
            // Clicking inactive persona activates it
            this.setActivePersona(personaId);
        }
    }
    
    setActivePersona(personaId) {
        // Clear previous active state
        this.bubbles.forEach(bubble => {
            bubble.setActive(false);
        });
        
        // Set new active state
        const activeBubble = this.bubbles.find(b => b.persona.id === personaId);
        if (activeBubble) {
            activeBubble.setActive(true);
            this.activePersonaId = personaId;
            
            // Save to localStorage
            // Requirements: 3.5
            localStorage.setItem('activePersonaId', personaId);
        }
    }
    
    clearActivePersona() {
        this.bubbles.forEach(bubble => {
            bubble.setActive(false);
        });
        this.activePersonaId = null;
        
        // Clear from localStorage
        // Requirements: 3.5
        localStorage.removeItem('activePersonaId');
    }
    
    restoreActivePersona() {
        // Restore active persona from localStorage on application load
        // Requirements: 3.5
        const savedPersonaId = localStorage.getItem('activePersonaId');
        if (savedPersonaId) {
            // Check if the saved persona still exists
            const personaExists = this.personas.some(p => p.id === savedPersonaId);
            if (personaExists) {
                this.setActivePersona(savedPersonaId);
            } else {
                // Clean up invalid persona ID from localStorage
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

// Cache DOM elements to avoid repeated queries
let messageInput, sendBtn, messagesContainer, modelSelect, newChatBtn, hamburgerBtn, sidebar, backdrop, conversationList, inputContainer;

// Persona system state
let personaBubbleContainer = null;
let personas = [];

function initDOMElements() {
    messageInput = document.getElementById('message-input');
    sendBtn = document.getElementById('send-btn');
    messagesContainer = document.getElementById('messages');
    modelSelect = document.getElementById('model-select');
    newChatBtn = document.getElementById('new-chat-btn');
    hamburgerBtn = document.getElementById('hamburger-btn');
    sidebar = document.querySelector('.sidebar');
    backdrop = document.querySelector('.backdrop');
    conversationList = document.getElementById('conversation-list');
    inputContainer = document.querySelector('.input-container');
    
    console.log('DOM Elements initialized:', {
        messageInput: !!messageInput,
        sendBtn: !!sendBtn,
        messagesContainer: !!messagesContainer,
        modelSelect: !!modelSelect,
        newChatBtn: !!newChatBtn,
        hamburgerBtn: !!hamburgerBtn,
        sidebar: !!sidebar,
        backdrop: !!backdrop,
        conversationList: !!conversationList,
        inputContainer: !!inputContainer
    });
}

// Load personas from backend
async function loadPersonas() {
    try {
        personas = await invoke('get_personas');
        console.log('Loaded personas:', personas);
        return personas;
    } catch (error) {
        console.error('Failed to load personas:', error);
        return [];
    }
}

// Initialize persona system
async function initPersonaSystem() {
    try {
        // Load personas from backend
        await loadPersonas();
        
        if (personas.length > 0 && inputContainer) {
            // Create and render persona bubble container
            personaBubbleContainer = new PersonaBubbleContainer(personas, inputContainer);
            personaBubbleContainer.render();
            
            // Restore active persona from localStorage
            // Requirements: 3.5
            personaBubbleContainer.restoreActivePersona();
            
            console.log('Persona system initialized with', personas.length, 'personas');
        } else {
            console.log('No personas available or input container not found');
        }
    } catch (error) {
        console.error('Failed to initialize persona system:', error);
    }
}

// Initialize app
async function init() {
    console.log('Initializing Prometheus...');
    
    // Initialize DOM elements first
    initDOMElements();
    
    // Load models
    await loadModels();
    
    // Initialize persona system
    await initPersonaSystem();
    
    // Load conversations list
    await loadConversations();
    
    // Load chat history
    await loadChatHistory();
    
    // Set up event listeners
    const inputForm = document.querySelector('.input-container');
    inputForm.addEventListener('submit', (e) => {
        e.preventDefault();
        sendMessage();
    });
    
    messageInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            sendMessage();
        }
    });
    
    // Add input validation listeners
    messageInput.addEventListener('input', updateSendButtonState);
    messageInput.addEventListener('keyup', updateSendButtonState);
    
    newChatBtn.addEventListener('click', newChat);
    modelSelect.addEventListener('change', (e) => {
        currentModel = e.target.value;
    });
    
    hamburgerBtn.addEventListener('click', toggleSidebar);
    
    // Add keyboard shortcut for sidebar toggle (Ctrl/Cmd + B)
    document.addEventListener('keydown', (e) => {
        if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
            e.preventDefault();
            toggleSidebar();
        }
        
        // Escape key closes sidebar
        if (e.key === 'Escape' && sidebarOpen) {
            closeSidebar();
        }
    });
    
    // Add animation event listeners
    sidebar.addEventListener('transitionend', (e) => {
        if (e.propertyName === 'transform') {
            isAnimating = false;
        }
    });
    
    // Add click-outside detection
    document.addEventListener('click', handleClickOutside);
    
    // Add backdrop click handler
    backdrop.addEventListener('click', () => {
        if (sidebarOpen) {
            closeSidebar();
        }
    });
    
    // Add window resize handler with debouncing
    let resizeTimeout;
    window.addEventListener('resize', () => {
        // Debounce resize events to minimize reflows
        clearTimeout(resizeTimeout);
        resizeTimeout = setTimeout(() => {
            handleResize();
        }, 150);
    }, { passive: true });
    
    // Initialize send button state
    updateSendButtonState();
    
    console.log('Prometheus initialized');
    
    // Add test button for debugging (remove in production)
    if (window.location.search.includes('debug')) {
        const testBtn = document.createElement('button');
        testBtn.textContent = 'Test Message';
        testBtn.style.cssText = 'position: fixed; top: 80px; right: 20px; z-index: 10000; padding: 10px; background: #00ff88; color: #0a0a0a; border: none; border-radius: 6px; cursor: pointer;';
        testBtn.onclick = () => {
            console.log('Test button clicked');
            addMessageToUI('user', 'Test user message');
            addMessageToUI('assistant', 'Test assistant response');
        };
        document.body.appendChild(testBtn);
    }
}

function toggleSidebar() {
    console.log('toggleSidebar called, isAnimating:', isAnimating, 'sidebarOpen:', sidebarOpen);
    if (isAnimating) return;
    
    if (sidebarOpen) {
        closeSidebar();
    } else {
        openSidebar();
    }
}

function openSidebar() {
    console.log('openSidebar called');
    if (isAnimating || sidebarOpen) {
        console.log('Skipping openSidebar - isAnimating:', isAnimating, 'sidebarOpen:', sidebarOpen);
        return;
    }
    
    isAnimating = true;
    sidebarOpen = true;
    sidebar.setAttribute('data-state', 'open');
    hamburgerBtn.classList.add('active');
    hamburgerBtn.setAttribute('aria-expanded', 'true');
    
    // Show backdrop on mobile
    backdrop.setAttribute('data-visible', 'true');
    
    console.log('Sidebar opened - data-state:', sidebar.getAttribute('data-state'));
    console.log('Sidebar computed transform:', window.getComputedStyle(sidebar).transform);
}

function closeSidebar() {
    console.log('closeSidebar called');
    if (isAnimating || !sidebarOpen) return;
    
    isAnimating = true;
    sidebarOpen = false;
    sidebar.setAttribute('data-state', 'closed');
    hamburgerBtn.classList.remove('active');
    hamburgerBtn.setAttribute('aria-expanded', 'false');
    
    // Hide backdrop
    backdrop.setAttribute('data-visible', 'false');
    console.log('Sidebar closed');
}

function handleClickOutside(event) {
    // Early return for performance - only handle clicks when sidebar is open
    if (!sidebarOpen) return;
    
    // Use event.target for efficient DOM checks
    const target = event.target;
    
    // Don't close if clicking inside the sidebar or hamburger button
    if (sidebar.contains(target) || hamburgerBtn.contains(target)) return;
    
    // Close sidebar for clicks outside
    closeSidebar();
}

function handleResize() {
    // Recalculate layout on resize
    // The sidebar state is maintained across breakpoints
    // CSS media queries handle the width changes automatically
    
    // Update backdrop visibility based on current state and screen size
    if (sidebarOpen) {
        const isMobile = window.innerWidth < 768;
        if (isMobile) {
            backdrop.setAttribute('data-visible', 'true');
        } else {
            backdrop.setAttribute('data-visible', 'false');
        }
    }
}

function updateSendButtonState() {
    const message = messageInput.value.trim();
    const isEmpty = message === '';
    
    // Disable button if input is empty or if loading
    sendBtn.disabled = isEmpty || isLoading;
}

async function loadModels() {
    try {
        const models = await invoke('get_models');
        modelSelect.innerHTML = '';
        
        if (models && models.length > 0) {
            models.forEach(model => {
                const option = document.createElement('option');
                option.value = model;
                option.textContent = model;
                modelSelect.appendChild(option);
            });
            currentModel = models[0];
            console.log('Loaded models:', models, 'Selected:', currentModel);
        } else {
            modelSelect.innerHTML = '<option>No models available</option>';
            console.warn('No models returned from Ollama');
        }
    } catch (error) {
        console.error('Failed to load models:', error);
        modelSelect.innerHTML = '<option>Ollama not running</option>';
        // Show a user-friendly message
        const errorMsg = document.createElement('div');
        errorMsg.style.cssText = 'position: fixed; top: 80px; left: 50%; transform: translateX(-50%); background: #ff4444; color: white; padding: 12px 20px; border-radius: 8px; z-index: 10000; font-size: 14px;';
        errorMsg.textContent = '⚠️ Cannot connect to Ollama. Make sure Ollama is running.';
        document.body.appendChild(errorMsg);
        setTimeout(() => errorMsg.remove(), 5000);
    }
}

async function loadConversations() {
    try {
        // Read the metadata file directly
        const fs = window.__TAURI__.fs;
        const metadataContent = await fs.readTextFile('conversations/metadata.json');
        const metadata = JSON.parse(metadataContent);
        
        conversations = metadata.conversations || [];
        renderConversationList();
    } catch (error) {
        console.error('Failed to load conversations:', error);
        // If file doesn't exist or error, just show empty list
        conversations = [];
        renderConversationList();
    }
}

function renderConversationList() {
    // Use DocumentFragment for better performance when adding multiple elements
    const fragment = document.createDocumentFragment();
    conversationList.innerHTML = '';
    
    if (conversations.length === 0) {
        conversationList.innerHTML = '<div style="color: var(--text-secondary); font-size: 12px; padding: 10px; text-align: center;">No conversations yet</div>';
        return;
    }
    
    conversations.forEach(conv => {
        const item = document.createElement('button');
        item.className = 'conversation-item';
        item.textContent = conv.name || conv.preview || 'Untitled';
        item.dataset.conversationId = conv.id;
        item.setAttribute('role', 'listitem');
        item.setAttribute('aria-label', `Load conversation: ${conv.name || conv.preview || 'Untitled'}`);
        
        // Highlight active conversation
        if (conv.id === currentConversationId) {
            item.classList.add('active');
            item.setAttribute('aria-current', 'true');
        }
        
        // Add click handler
        item.addEventListener('click', () => selectConversation(conv.id));
        
        fragment.appendChild(item);
    });
    
    // Single DOM update instead of multiple
    conversationList.appendChild(fragment);
}

async function selectConversation(conversationId) {
    try {
        // Load the conversation file
        const fs = window.__TAURI__.fs;
        const conversationContent = await fs.readTextFile(`conversations/${conversationId}.json`);
        const conversation = JSON.parse(conversationContent);
        
        // Update current conversation ID
        currentConversationId = conversationId;
        
        // Clear and load messages
        messagesContainer.innerHTML = '';
        
        if (conversation.messages && conversation.messages.length > 0) {
            conversation.messages.forEach(msg => {
                addMessageToUI(msg.role, msg.content);
            });
        } else {
            showEmptyState();
        }
        
        // Update highlighting in sidebar
        renderConversationList();
        
        // Close sidebar after selection
        closeSidebar();
        
    } catch (error) {
        console.error('Failed to load conversation:', error);
        alert('Failed to load conversation');
    }
}

async function loadChatHistory() {
    try {
        const history = await invoke('get_chat_history');
        messagesContainer.innerHTML = '';
        
        if (history && history.length > 0) {
            history.forEach(msg => {
                addMessageToUI(msg.role, msg.content);
            });
        } else {
            showEmptyState();
        }
    } catch (error) {
        console.error('Failed to load chat history:', error);
        showEmptyState();
    }
}

function showEmptyState() {
    messagesContainer.innerHTML = `
        <div class="empty-state">
            <pre>
   ________  ________  ________  ________  ________  ________  ________  ________  ________  ________ 
  /        \\/        \\/        \\/        \\/        \\/        \\/    /   \\/        \\/    /   \\/        \\
 /         /         /         /         /         /        _/         /         /         /        _/
//      __/        _/         /         /        _//       //         /        _/         /-        / 
\\\\_____/  \\____/___/\\________/\\__/__/__/\\________/ \\______/ \\___/____/\\________/\\________/\\________/  
            </pre>
            <p>Start a conversation...</p>
        </div>
    `;
}

async function sendMessage() {
    console.log('sendMessage called');
    if (isLoading) {
        console.log('Already loading, skipping');
        return;
    }
    
    const message = messageInput.value.trim();
    console.log('Message:', message);
    if (!message) {
        console.log('Empty message, skipping');
        return;
    }
    
    // Clear input
    messageInput.value = '';
    
    // Add user message to UI
    addMessageToUI('user', message);
    
    // Set loading state
    isLoading = true;
    sendBtn.disabled = true;
    sendBtn.classList.add('loading');
    
    // Disable persona bubbles during message sending
    if (personaBubbleContainer) {
        personaBubbleContainer.setDisabled(true);
    }
    
    // Add empty assistant message for streaming
    const assistantMsg = addMessageToUI('assistant', '');
    
    // Retrieve active persona and extract system prompt
    // Requirements: 3.1, 2.6
    const activePersona = personaBubbleContainer ? personaBubbleContainer.getActivePersona() : null;
    const systemPrompt = activePersona ? activePersona.system_prompt : null;
    
    console.log('Active persona:', activePersona ? activePersona.name : 'None');
    console.log('System prompt:', systemPrompt ? 'Present' : 'None');
    
    // Generate unique request ID
    const requestId = `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    // Set up event listeners for streaming
    const { listen } = window.__TAURI__.event;
    
    let streamContent = '';
    
    const unlistenToken = await listen('stream-token', (event) => {
        if (event.payload.request_id === requestId) {
            streamContent += event.payload.token;
            assistantMsg.textContent = streamContent;
            // Auto-scroll to bottom
            messagesContainer.scrollTop = messagesContainer.scrollHeight;
        }
    });
    
    const unlistenDone = await listen('stream-done', (event) => {
        if (event.payload.request_id === requestId) {
            console.log('Stream completed');
            isLoading = false;
            sendBtn.classList.remove('loading');
            updateSendButtonState();
            
            // Re-enable persona bubbles after message completes
            if (personaBubbleContainer) {
                personaBubbleContainer.setDisabled(false);
            }
            
            unlistenToken();
            unlistenDone();
            unlistenError();
        }
    });
    
    const unlistenError = await listen('stream-error', (event) => {
        if (event.payload.request_id === requestId) {
            console.error('Stream error:', event.payload.error);
            assistantMsg.textContent = `Error: ${event.payload.error}`;
            assistantMsg.style.color = '#ff4444';
            isLoading = false;
            sendBtn.classList.remove('loading');
            updateSendButtonState();
            
            // Re-enable persona bubbles after error
            if (personaBubbleContainer) {
                personaBubbleContainer.setDisabled(false);
            }
            
            unlistenToken();
            unlistenDone();
            unlistenError();
        }
    });
    
    try {
        console.log('Starting stream with model:', currentModel);
        // Start streaming with system prompt if active persona exists
        // Requirements: 3.1, 2.6
        await invoke('send_message_stream', {
            prompt: message,
            model: currentModel || 'llama2',
            requestId: requestId,
            systemPrompt: systemPrompt // Pass system prompt (null/undefined if no active persona)
        });
    } catch (error) {
        console.error('Error starting stream:', error);
        assistantMsg.textContent = `Error: ${error}`;
        assistantMsg.style.color = '#ff4444';
        isLoading = false;
        sendBtn.classList.remove('loading');
        updateSendButtonState();
        
        // Re-enable persona bubbles after error
        if (personaBubbleContainer) {
            personaBubbleContainer.setDisabled(false);
        }
        
        unlistenToken();
        unlistenDone();
        unlistenError();
    }
}

function addMessageToUI(role, content) {
    // Remove empty state if present
    const emptyState = messagesContainer.querySelector('.empty-state');
    if (emptyState) {
        emptyState.remove();
    }
    
    const messageDiv = document.createElement('div');
    messageDiv.className = `message ${role}`;
    messageDiv.setAttribute('role', 'article');
    messageDiv.setAttribute('aria-label', `${role === 'user' ? 'Your' : 'Assistant'} message`);
    
    const contentDiv = document.createElement('div');
    contentDiv.className = 'message-content';
    contentDiv.textContent = content;
    
    messageDiv.appendChild(contentDiv);
    messagesContainer.appendChild(messageDiv);
    
    // Scroll to bottom
    messagesContainer.scrollTop = messagesContainer.scrollHeight;
    
    return contentDiv;
}

async function newChat() {
    try {
        // Call backend to create new conversation
        await invoke('new_conversation');
        
        // Clear current conversation ID
        currentConversationId = null;
        
        // Clear persona state on new conversation
        // Requirements: 3.5
        if (personaBubbleContainer) {
            personaBubbleContainer.clearActivePersona();
        }
        
        // Clear chat history display
        messagesContainer.innerHTML = '';
        showEmptyState();
        
        // Reload conversations list to show the new one
        await loadConversations();
        
        // Close sidebar after creation
        closeSidebar();
        
    } catch (error) {
        console.error('Failed to create new chat:', error);
    }
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
