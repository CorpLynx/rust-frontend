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
let settingsBtn, settingsModal, settingsCloseBtn, endpointDialog, endpointDialogCloseBtn;
let modeLocalRadio, modeRemoteRadio, endpointsList, addEndpointBtn;
let endpointForm, testEndpointBtn, cancelEndpointBtn, saveEndpointBtn;

// Persona system state
let personaBubbleContainer = null;
let personas = [];

// Settings state
let currentConnectionMode = 'local';
let remoteEndpoints = [];
let activeRemoteEndpointId = null;
let editingEndpointId = null;

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
    
    // Settings elements
    settingsBtn = document.getElementById('settings-btn');
    settingsModal = document.getElementById('settings-modal');
    settingsCloseBtn = document.getElementById('settings-close-btn');
    endpointDialog = document.getElementById('endpoint-dialog');
    endpointDialogCloseBtn = document.getElementById('endpoint-dialog-close-btn');
    modeLocalRadio = document.getElementById('mode-local');
    modeRemoteRadio = document.getElementById('mode-remote');
    endpointsList = document.getElementById('endpoints-list');
    addEndpointBtn = document.getElementById('add-endpoint-btn');
    endpointForm = document.getElementById('endpoint-form');
    testEndpointBtn = document.getElementById('test-endpoint-btn');
    cancelEndpointBtn = document.getElementById('cancel-endpoint-btn');
    saveEndpointBtn = document.getElementById('save-endpoint-btn');
    
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
        inputContainer: !!inputContainer,
        settingsBtn: !!settingsBtn,
        settingsModal: !!settingsModal
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
    
    // Settings event listeners (with null checks)
    if (settingsBtn) {
        settingsBtn.addEventListener('click', openSettings);
    }
    if (settingsCloseBtn) {
        settingsCloseBtn.addEventListener('click', closeSettings);
    }
    if (settingsModal) {
        settingsModal.addEventListener('click', (e) => {
            if (e.target === settingsModal) {
                closeSettings();
            }
        });
    }
    
    // Connection mode toggle
    if (modeLocalRadio) {
        modeLocalRadio.addEventListener('change', () => {
            if (modeLocalRadio.checked) {
                setConnectionMode('local');
            }
        });
    }
    
    if (modeRemoteRadio) {
        modeRemoteRadio.addEventListener('change', () => {
            if (modeRemoteRadio.checked) {
                setConnectionMode('remote');
            }
        });
    }
    
    // Endpoint management
    if (addEndpointBtn) {
        addEndpointBtn.addEventListener('click', openAddEndpointDialog);
    }
    if (endpointDialogCloseBtn) {
        endpointDialogCloseBtn.addEventListener('click', closeEndpointDialog);
    }
    if (cancelEndpointBtn) {
        cancelEndpointBtn.addEventListener('click', closeEndpointDialog);
    }
    if (endpointDialog) {
        endpointDialog.addEventListener('click', (e) => {
            if (e.target === endpointDialog) {
                closeEndpointDialog();
            }
        });
    }
    
    if (endpointForm) {
        endpointForm.addEventListener('submit', (e) => {
            e.preventDefault();
            saveEndpoint();
        });
    }
    
    if (testEndpointBtn) {
        testEndpointBtn.addEventListener('click', testEndpointConnection);
    }
    
    // Add HTTPS checkbox change handler to show warning
    const endpointHttpsCheckbox = document.getElementById('endpoint-https');
    if (endpointHttpsCheckbox) {
        endpointHttpsCheckbox.addEventListener('change', updateHttpsWarning);
    }
    
    // Add input validation listeners for real-time feedback
    const endpointNameInput = document.getElementById('endpoint-name');
    const endpointHostInput = document.getElementById('endpoint-host');
    const endpointPortInput = document.getElementById('endpoint-port');
    
    if (endpointNameInput) {
        endpointNameInput.addEventListener('input', () => validateEndpointName());
    }
    if (endpointHostInput) {
        endpointHostInput.addEventListener('input', () => validateEndpointHost());
    }
    if (endpointPortInput) {
        endpointPortInput.addEventListener('input', () => validateEndpointPort());
    }
    
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
    
    // Load settings (non-blocking, don't fail initialization if settings fail)
    loadSettings().catch(error => {
        console.error('Failed to load settings during init:', error);
        // Continue initialization even if settings fail
    });
    
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

// Settings Functions

async function loadSettings() {
    try {
        // Load connection mode
        const mode = await invoke('get_connection_mode');
        currentConnectionMode = mode.toLowerCase();
        
        if (currentConnectionMode === 'local') {
            modeLocalRadio.checked = true;
        } else {
            modeRemoteRadio.checked = true;
        }
        
        // Load remote endpoints
        remoteEndpoints = await invoke('list_remote_endpoints');
        
        // Load active endpoint
        const activeEndpoint = await invoke('get_active_endpoint');
        if (activeEndpoint && activeEndpoint.id) {
            activeRemoteEndpointId = activeEndpoint.id;
        }
        
        renderEndpointsList();
        
        console.log('Settings loaded:', { mode: currentConnectionMode, endpoints: remoteEndpoints.length });
    } catch (error) {
        console.error('Failed to load settings:', error);
        // Set defaults if loading fails
        currentConnectionMode = 'local';
        remoteEndpoints = [];
        activeRemoteEndpointId = null;
        if (modeLocalRadio) modeLocalRadio.checked = true;
        if (endpointsList) {
            endpointsList.innerHTML = `
                <div class="empty-endpoints-message">
                    No remote endpoints configured. Click "Add Endpoint" to get started.
                </div>
            `;
        }
    }
}

function openSettings() {
    if (!settingsModal) {
        console.error('Settings modal not found');
        return;
    }
    settingsModal.setAttribute('aria-hidden', 'false');
    // Reload settings when opening (non-blocking)
    loadSettings().catch(error => {
        console.error('Failed to reload settings:', error);
    });
}

function closeSettings() {
    if (!settingsModal) {
        console.error('Settings modal not found');
        return;
    }
    settingsModal.setAttribute('aria-hidden', 'true');
}

async function setConnectionMode(mode) {
    const operationId = 'set-connection-mode';
    loadingManager.startOperation(operationId);
    
    try {
        await invoke('set_connection_mode', { mode: mode === 'local' ? 'Local' : 'Remote' });
        currentConnectionMode = mode;
        console.log('Connection mode set to:', mode);
        showToast(`Connection mode changed to ${mode}`, 'success');
    } catch (error) {
        console.error('Failed to set connection mode:', error);
        showToast(`Failed to change connection mode: ${formatErrorMessage(error.toString())}`, 'error', 5000);
        
        // Revert radio button selection
        if (mode === 'local') {
            modeRemoteRadio.checked = true;
        } else {
            modeLocalRadio.checked = true;
        }
    } finally {
        loadingManager.endOperation(operationId);
    }
}

function renderEndpointsList() {
    if (!endpointsList) {
        console.error('Endpoints list element not found');
        return;
    }
    
    if (remoteEndpoints.length === 0) {
        endpointsList.innerHTML = `
            <div class="empty-endpoints-message">
                No remote endpoints configured. Click "Add Endpoint" to get started.
            </div>
        `;
        return;
    }
    
    endpointsList.innerHTML = '';
    
    remoteEndpoints.forEach(endpoint => {
        const item = document.createElement('div');
        item.className = 'endpoint-item';
        if (endpoint.id === activeRemoteEndpointId) {
            item.classList.add('active');
        }
        item.setAttribute('role', 'listitem');
        
        // Radio button for selection
        const radioDiv = document.createElement('div');
        radioDiv.className = 'endpoint-radio';
        const radio = document.createElement('input');
        radio.type = 'radio';
        radio.name = 'active-endpoint';
        radio.value = endpoint.id;
        radio.checked = endpoint.id === activeRemoteEndpointId;
        radio.addEventListener('change', () => selectEndpoint(endpoint.id));
        radioDiv.appendChild(radio);
        
        // Endpoint info
        const infoDiv = document.createElement('div');
        infoDiv.className = 'endpoint-info';
        
        const nameDiv = document.createElement('div');
        nameDiv.className = 'endpoint-name';
        nameDiv.textContent = endpoint.name;
        
        const addressDiv = document.createElement('div');
        addressDiv.className = 'endpoint-address';
        const protocol = endpoint.use_https ? 'https' : 'http';
        addressDiv.textContent = `${protocol}://${endpoint.host}:${endpoint.port}`;
        
        const statusDiv = document.createElement('div');
        statusDiv.className = 'endpoint-status';
        if (endpoint.last_tested) {
            if (endpoint.last_test_success) {
                statusDiv.classList.add('success');
                statusDiv.textContent = `✓ Last tested: ${formatTimestamp(endpoint.last_tested)}`;
            } else {
                statusDiv.classList.add('error');
                statusDiv.textContent = `✗ Last test failed: ${formatTimestamp(endpoint.last_tested)}`;
            }
        } else {
            statusDiv.classList.add('never-tested');
            statusDiv.textContent = 'Never tested';
        }
        
        infoDiv.appendChild(nameDiv);
        infoDiv.appendChild(addressDiv);
        
        // Display masked API key if present (Requirement 8.3)
        if (endpoint.api_key) {
            const apiKeyDiv = document.createElement('div');
            apiKeyDiv.className = 'endpoint-api-key';
            apiKeyDiv.textContent = 'API Key: ••••••••';
            apiKeyDiv.style.fontSize = '12px';
            apiKeyDiv.style.color = 'var(--text-secondary)';
            apiKeyDiv.style.marginTop = '4px';
            infoDiv.appendChild(apiKeyDiv);
        }
        
        infoDiv.appendChild(statusDiv);
        
        // Actions
        const actionsDiv = document.createElement('div');
        actionsDiv.className = 'endpoint-actions';
        
        const testBtn = document.createElement('button');
        testBtn.className = 'endpoint-action-btn test-btn';
        testBtn.textContent = 'Test';
        testBtn.addEventListener('click', () => testEndpoint(endpoint.id));
        
        const editBtn = document.createElement('button');
        editBtn.className = 'endpoint-action-btn';
        editBtn.textContent = 'Edit';
        editBtn.addEventListener('click', () => openEditEndpointDialog(endpoint));
        
        const deleteBtn = document.createElement('button');
        deleteBtn.className = 'endpoint-action-btn delete-btn';
        deleteBtn.textContent = 'Delete';
        deleteBtn.addEventListener('click', () => deleteEndpoint(endpoint.id));
        
        actionsDiv.appendChild(testBtn);
        actionsDiv.appendChild(editBtn);
        actionsDiv.appendChild(deleteBtn);
        
        item.appendChild(radioDiv);
        item.appendChild(infoDiv);
        item.appendChild(actionsDiv);
        
        endpointsList.appendChild(item);
    });
}

function formatTimestamp(timestamp) {
    try {
        const date = new Date(timestamp);
        const now = new Date();
        const diffMs = now - date;
        const diffMins = Math.floor(diffMs / 60000);
        
        if (diffMins < 1) return 'just now';
        if (diffMins < 60) return `${diffMins} minute${diffMins > 1 ? 's' : ''} ago`;
        
        const diffHours = Math.floor(diffMins / 60);
        if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
        
        const diffDays = Math.floor(diffHours / 24);
        return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
    } catch (error) {
        return 'unknown';
    }
}

async function selectEndpoint(endpointId) {
    const operationId = `select-endpoint-${endpointId}`;
    loadingManager.startOperation(operationId);
    
    try {
        await invoke('set_active_remote_endpoint', { endpointId });
        activeRemoteEndpointId = endpointId;
        
        const endpoint = remoteEndpoints.find(e => e.id === endpointId);
        const endpointName = endpoint ? endpoint.name : 'endpoint';
        
        renderEndpointsList();
        console.log('Active endpoint set to:', endpointId);
        showToast(`Active endpoint set to "${endpointName}"`, 'success');
    } catch (error) {
        console.error('Failed to set active endpoint:', error);
        showToast(`Failed to select endpoint: ${formatErrorMessage(error.toString())}`, 'error', 5000);
        
        // Reload to revert selection
        await loadSettings();
    } finally {
        loadingManager.endOperation(operationId);
    }
}

function openAddEndpointDialog() {
    editingEndpointId = null;
    document.getElementById('endpoint-dialog-title').textContent = 'Add Remote Endpoint';
    endpointForm.reset();
    clearFormErrors();
    updateHttpsWarning(); // Update warning based on default checkbox state
    endpointDialog.setAttribute('aria-hidden', 'false');
}

function openEditEndpointDialog(endpoint) {
    editingEndpointId = endpoint.id;
    document.getElementById('endpoint-dialog-title').textContent = 'Edit Remote Endpoint';
    
    document.getElementById('endpoint-name').value = endpoint.name;
    document.getElementById('endpoint-host').value = endpoint.host;
    document.getElementById('endpoint-port').value = endpoint.port;
    document.getElementById('endpoint-https').checked = endpoint.use_https;
    document.getElementById('endpoint-api-key').value = endpoint.api_key || '';
    
    clearFormErrors();
    updateHttpsWarning(); // Update warning based on loaded endpoint settings
    endpointDialog.setAttribute('aria-hidden', 'false');
}

function closeEndpointDialog() {
    endpointDialog.setAttribute('aria-hidden', 'true');
    endpointForm.reset();
    clearFormErrors();
    editingEndpointId = null;
}

function clearFormErrors() {
    document.getElementById('name-error').textContent = '';
    document.getElementById('host-error').textContent = '';
    document.getElementById('port-error').textContent = '';
    document.getElementById('api-key-error').textContent = '';
}

// Client-side validation functions (Requirements 1.2, 1.3)
function validateEndpointName() {
    const nameInput = document.getElementById('endpoint-name');
    const nameError = document.getElementById('name-error');
    const name = nameInput.value.trim();
    
    if (!name) {
        nameError.textContent = 'Name is required';
        return false;
    }
    
    if (name.length > 100) {
        nameError.textContent = 'Name must be 100 characters or less';
        return false;
    }
    
    nameError.textContent = '';
    return true;
}

function validateEndpointHost() {
    const hostInput = document.getElementById('endpoint-host');
    const hostError = document.getElementById('host-error');
    const host = hostInput.value.trim();
    
    if (!host) {
        hostError.textContent = 'Host is required';
        return false;
    }
    
    // Basic validation for IP address or hostname
    // IPv4 pattern
    const ipv4Pattern = /^(\d{1,3}\.){3}\d{1,3}$/;
    // IPv6 pattern (simplified)
    const ipv6Pattern = /^([0-9a-fA-F]{0,4}:){2,7}[0-9a-fA-F]{0,4}$/;
    // Hostname pattern
    const hostnamePattern = /^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$/;
    
    const isValidIpv4 = ipv4Pattern.test(host);
    const isValidIpv6 = ipv6Pattern.test(host);
    const isValidHostname = hostnamePattern.test(host);
    
    if (!isValidIpv4 && !isValidIpv6 && !isValidHostname) {
        hostError.textContent = 'Invalid IP address or hostname format';
        return false;
    }
    
    // Additional IPv4 validation - check each octet is 0-255
    if (isValidIpv4) {
        const octets = host.split('.');
        for (const octet of octets) {
            const num = parseInt(octet, 10);
            if (num < 0 || num > 255) {
                hostError.textContent = 'Invalid IPv4 address (octets must be 0-255)';
                return false;
            }
        }
    }
    
    hostError.textContent = '';
    return true;
}

function validateEndpointPort() {
    const portInput = document.getElementById('endpoint-port');
    const portError = document.getElementById('port-error');
    const portValue = portInput.value.trim();
    
    if (!portValue) {
        portError.textContent = 'Port is required';
        return false;
    }
    
    const port = parseInt(portValue, 10);
    
    if (isNaN(port)) {
        portError.textContent = 'Port must be a number';
        return false;
    }
    
    if (port < 1 || port > 65535) {
        portError.textContent = 'Port must be between 1 and 65535';
        return false;
    }
    
    portError.textContent = '';
    return true;
}

// Show/hide HTTP warning based on HTTPS checkbox (Requirement 3.5)
function updateHttpsWarning() {
    const httpsCheckbox = document.getElementById('endpoint-https');
    const httpWarning = document.getElementById('http-warning');
    
    if (!httpsCheckbox || !httpWarning) return;
    
    // Show warning when HTTPS is NOT checked (i.e., using HTTP)
    if (!httpsCheckbox.checked) {
        httpWarning.style.display = 'block';
    } else {
        httpWarning.style.display = 'none';
    }
}

async function saveEndpoint() {
    clearFormErrors();
    
    // Run all validations (inline validation errors shown)
    const isNameValid = validateEndpointName();
    const isHostValid = validateEndpointHost();
    const isPortValid = validateEndpointPort();
    
    // If any validation fails, stop and show toast
    if (!isNameValid || !isHostValid || !isPortValid) {
        showToast('Please fix validation errors before saving', 'error');
        return;
    }
    
    const name = document.getElementById('endpoint-name').value.trim();
    const host = document.getElementById('endpoint-host').value.trim();
    const port = parseInt(document.getElementById('endpoint-port').value);
    const useHttps = document.getElementById('endpoint-https').checked;
    const apiKey = document.getElementById('endpoint-api-key').value.trim() || null;
    
    // Start loading state
    const operationId = 'save-endpoint';
    loadingManager.startOperation(operationId, saveEndpointBtn);
    const originalText = saveEndpointBtn.textContent;
    saveEndpointBtn.textContent = 'Saving...';
    
    try {
        if (editingEndpointId) {
            // Update existing endpoint
            await invoke('update_remote_endpoint', {
                endpointId: editingEndpointId,
                name,
                host,
                port,
                useHttps,
                apiKey
            });
            console.log('Endpoint updated:', editingEndpointId);
            showToast(`Endpoint "${name}" updated successfully`, 'success');
        } else {
            // Add new endpoint
            await invoke('add_remote_endpoint', {
                name,
                host,
                port,
                useHttps,
                apiKey
            });
            console.log('Endpoint added');
            showToast(`Endpoint "${name}" added successfully`, 'success');
        }
        
        // Reload endpoints list
        await loadSettings();
        closeEndpointDialog();
    } catch (error) {
        console.error('Failed to save endpoint:', error);
        
        // Display inline validation errors where appropriate
        const errorMessage = error.toString();
        if (errorMessage.toLowerCase().includes('host') || errorMessage.toLowerCase().includes('ip')) {
            document.getElementById('host-error').textContent = formatErrorMessage(error.toString());
        } else if (errorMessage.toLowerCase().includes('port')) {
            document.getElementById('port-error').textContent = formatErrorMessage(error.toString());
        } else {
            // Show generic error as toast
            showToast(`Failed to save endpoint: ${formatErrorMessage(error.toString())}`, 'error', 5000);
        }
    } finally {
        loadingManager.endOperation(operationId, saveEndpointBtn);
        saveEndpointBtn.textContent = originalText;
    }
}

async function testEndpointConnection() {
    const host = document.getElementById('endpoint-host').value.trim();
    const port = parseInt(document.getElementById('endpoint-port').value);
    const useHttps = document.getElementById('endpoint-https').checked;
    
    if (!host || !port) {
        showToast('Please enter host and port before testing', 'error');
        return;
    }
    
    // Start loading state (Requirement 4.3)
    const operationId = 'test-endpoint-dialog';
    loadingManager.startOperation(operationId, testEndpointBtn);
    const originalText = testEndpointBtn.textContent;
    testEndpointBtn.textContent = 'Testing...';
    
    // Remove any existing test result
    const existingResult = endpointForm.querySelector('.connection-test-result');
    if (existingResult) {
        existingResult.remove();
    }
    
    try {
        const result = await invoke('test_remote_endpoint', {
            host,
            port,
            useHttps
        });
        
        // Display connection test result (Requirements 4.3, 4.4)
        const resultDiv = showConnectionTestResult(result, endpointForm);
        
        // Also show toast notification
        if (result.success) {
            showToast(`Connection successful! (${result.response_time_ms}ms)`, 'success');
        } else {
            showToast('Connection test failed', 'error');
        }
    } catch (error) {
        console.error('Test connection error:', error);
        
        // Display error with helpful message (Requirements 7.1, 7.2, 7.3, 7.4)
        const result = {
            success: false,
            error_message: error.toString()
        };
        showConnectionTestResult(result, endpointForm);
        showToast('Connection test failed', 'error');
    } finally {
        loadingManager.endOperation(operationId, testEndpointBtn);
        testEndpointBtn.textContent = originalText;
    }
}

async function testEndpoint(endpointId) {
    const endpoint = remoteEndpoints.find(e => e.id === endpointId);
    if (!endpoint) return;
    
    // Find the test button for this endpoint
    const endpointItem = Array.from(endpointsList.children).find(
        item => item.querySelector('input[type="radio"]')?.value === endpointId
    );
    const testBtn = endpointItem?.querySelector('.test-btn');
    
    // Start loading state (Requirement 4.3)
    const operationId = `test-endpoint-${endpointId}`;
    if (testBtn) {
        loadingManager.startOperation(operationId, testBtn);
        testBtn.textContent = 'Testing...';
    }
    
    try {
        const result = await invoke('test_remote_endpoint', {
            host: endpoint.host,
            port: endpoint.port,
            useHttps: endpoint.use_https
        });
        
        // Display connection test result (Requirements 4.3, 4.4)
        if (result.success) {
            showToast(`${endpoint.name}: Connection successful! (${result.response_time_ms}ms)`, 'success');
        } else {
            const errorMessage = formatErrorMessage(result.error_message || 'Unknown error');
            showToast(`${endpoint.name}: ${errorMessage}`, 'error', 5000);
        }
        
        // Reload to update status
        await loadSettings();
    } catch (error) {
        console.error('Test connection error:', error);
        
        // Display error with helpful message (Requirements 7.1, 7.2, 7.3, 7.4)
        const errorMessage = formatErrorMessage(error.toString());
        showToast(`${endpoint.name}: ${errorMessage}`, 'error', 5000);
    } finally {
        if (testBtn) {
            loadingManager.endOperation(operationId, testBtn);
            testBtn.textContent = 'Test';
        }
    }
}

async function deleteEndpoint(endpointId) {
    const endpoint = remoteEndpoints.find(e => e.id === endpointId);
    if (!endpoint) return;
    
    if (!confirm(`Are you sure you want to delete "${endpoint.name}"?`)) {
        return;
    }
    
    // Start loading state
    const operationId = `delete-endpoint-${endpointId}`;
    loadingManager.startOperation(operationId);
    
    try {
        await invoke('remove_remote_endpoint', { endpointId });
        console.log('Endpoint deleted:', endpointId);
        
        // Show success toast notification
        showToast(`Endpoint "${endpoint.name}" deleted successfully`, 'success');
        
        // If we deleted the active endpoint, clear it
        if (endpointId === activeRemoteEndpointId) {
            activeRemoteEndpointId = null;
        }
        
        // Reload endpoints list
        await loadSettings();
    } catch (error) {
        console.error('Failed to delete endpoint:', error);
        showToast(`Failed to delete endpoint: ${formatErrorMessage(error.toString())}`, 'error', 5000);
    } finally {
        loadingManager.endOperation(operationId);
    }
}

// Toast notification system
function showToast(message, type = 'info', duration = 3000) {
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    toast.setAttribute('role', 'alert');
    toast.setAttribute('aria-live', 'polite');
    
    document.body.appendChild(toast);
    
    // Trigger animation
    setTimeout(() => {
        toast.classList.add('show');
    }, 10);
    
    // Remove after duration
    setTimeout(() => {
        toast.classList.remove('show');
        setTimeout(() => {
            toast.remove();
        }, 300);
    }, duration);
}

// Connection test result display
function showConnectionTestResult(result, container = null) {
    const resultDiv = document.createElement('div');
    resultDiv.className = `connection-test-result ${result.success ? 'success' : 'error'}`;
    resultDiv.setAttribute('role', 'status');
    resultDiv.setAttribute('aria-live', 'polite');
    
    if (result.success) {
        resultDiv.innerHTML = `
            <span class="result-icon">✓</span>
            <span class="result-message">Connection successful!</span>
            <span class="result-detail">Response time: ${result.response_time_ms}ms</span>
        `;
    } else {
        const errorMessage = formatErrorMessage(result.error_message || 'Unknown error');
        resultDiv.innerHTML = `
            <span class="result-icon">✗</span>
            <span class="result-message">Connection failed</span>
            <span class="result-detail">${errorMessage}</span>
        `;
    }
    
    if (container) {
        // Remove any existing result
        const existingResult = container.querySelector('.connection-test-result');
        if (existingResult) {
            existingResult.remove();
        }
        container.appendChild(resultDiv);
        
        // Auto-remove after 10 seconds
        setTimeout(() => {
            resultDiv.remove();
        }, 10000);
    }
    
    return resultDiv;
}

// Format error messages with helpful context (Requirements 7.1, 7.2, 7.3, 7.4)
function formatErrorMessage(error) {
    const errorStr = error.toString().toLowerCase();
    
    // Network timeout (Requirement 7.1)
    if (errorStr.includes('timeout') || errorStr.includes('timed out')) {
        return 'Server unreachable - connection timed out. Please check if the server is running and accessible.';
    }
    
    // Connection refused (Requirement 7.2)
    if (errorStr.includes('connection refused') || errorStr.includes('refused')) {
        return 'Server not accepting connections. Please verify the server is running and the port is correct.';
    }
    
    // Invalid response (Requirement 7.3)
    if (errorStr.includes('invalid response') || errorStr.includes('protocol') || errorStr.includes('parse')) {
        return 'Invalid response from server - protocol mismatch. Please verify this is an Ollama server.';
    }
    
    // TLS/SSL errors (Requirement 7.4)
    if (errorStr.includes('tls') || errorStr.includes('ssl') || errorStr.includes('certificate') || errorStr.includes('cert')) {
        return 'TLS/SSL certificate error. The server certificate may be invalid, self-signed, or expired. Try using HTTP instead or install the proper certificate.';
    }
    
    // DNS resolution
    if (errorStr.includes('dns') || errorStr.includes('resolve') || errorStr.includes('name')) {
        return 'Cannot resolve hostname. Please check the host address is correct.';
    }
    
    // Network unreachable
    if (errorStr.includes('network') || errorStr.includes('unreachable')) {
        return 'Network unreachable. Please check your network connection and firewall settings.';
    }
    
    // Default: return original error with context
    return `${error}. Please check your connection settings and try again.`;
}

// Loading state manager
class LoadingStateManager {
    constructor() {
        this.activeOperations = new Set();
    }
    
    startOperation(operationId, element = null) {
        this.activeOperations.add(operationId);
        if (element) {
            element.classList.add('loading');
            element.disabled = true;
        }
    }
    
    endOperation(operationId, element = null) {
        this.activeOperations.delete(operationId);
        if (element) {
            element.classList.remove('loading');
            element.disabled = false;
        }
    }
    
    isLoading(operationId = null) {
        if (operationId) {
            return this.activeOperations.has(operationId);
        }
        return this.activeOperations.size > 0;
    }
}

const loadingManager = new LoadingStateManager();

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
