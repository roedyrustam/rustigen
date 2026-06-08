// State Management
let state = {
    sessions: [], // { id, title, messages: [] }
    activeSessionId: null,
    settings: {
        apiKey: "",
        model: "gemini-2.5-flash",
        temperature: 0.7
    }
};

// Elements
const chatMessages = document.getElementById("chat-messages");
const welcomeContainer = document.getElementById("welcome-container");
const chatForm = document.getElementById("chat-form");
const chatInput = document.getElementById("chat-input");
const chatList = document.getElementById("chat-list");
const newChatBtn = document.getElementById("new-chat-btn");
const clearChatBtn = document.getElementById("clear-chat-btn");
const currentChatTitle = document.getElementById("current-chat-title");
const statusBadge = document.getElementById("status-badge");
const statusText = document.getElementById("status-text");

// Settings Modal Elements
const settingsModal = document.getElementById("settings-modal");
const openSettingsBtn = document.getElementById("open-settings-btn");
const closeSettingsBtn = document.getElementById("close-settings-btn");
const saveSettingsBtn = document.getElementById("save-settings-btn");
const resetSettingsBtn = document.getElementById("reset-settings-btn");
const settingApiKey = document.getElementById("setting-api-key");
const settingModel = document.getElementById("setting-model");
const settingTempSlider = document.getElementById("setting-temp-slider");
const settingTempVal = document.getElementById("setting-temp-val");

// Initial Setup
document.addEventListener("DOMContentLoaded", () => {
    loadSettings();
    loadSessions();
    setupEventListeners();
    updateUIState();
});

// Event Listeners
function setupEventListeners() {
    // Form submission
    chatForm.addEventListener("submit", (e) => {
        e.preventDefault();
        sendMessage();
    });

    // Auto-resize input textarea
    chatInput.addEventListener("input", () => {
        chatInput.style.height = "auto";
        chatInput.style.height = (chatInput.scrollHeight) + "px";
    });

    // Send on Enter (but new line on Shift+Enter)
    chatInput.addEventListener("keydown", (e) => {
        if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            chatForm.requestSubmit();
        }
    });

    // Navigation / Chat Session Buttons
    newChatBtn.addEventListener("click", createNewSession);
    clearChatBtn.addEventListener("click", deleteActiveSession);

    // Modal Triggers
    openSettingsBtn.addEventListener("click", () => toggleModal(true));
    closeSettingsBtn.addEventListener("click", () => toggleModal(false));
    settingsModal.addEventListener("click", (e) => {
        if (e.target === settingsModal) toggleModal(false);
    });

    // Settings actions
    saveSettingsBtn.addEventListener("click", saveSettings);
    resetSettingsBtn.addEventListener("click", resetSettings);
    settingTempSlider.addEventListener("input", (e) => {
        settingTempVal.textContent = e.target.value;
    });
}

// Settings Handlers
function loadSettings() {
    const saved = localStorage.getItem("rust_agent_settings");
    if (saved) {
        state.settings = JSON.parse(saved);
    }
    
    // Populate form fields
    settingApiKey.value = state.settings.apiKey || "";
    settingModel.value = state.settings.model || "gemini-2.5-flash";
    settingTempSlider.value = state.settings.temperature || 0.7;
    settingTempVal.textContent = state.settings.temperature || 0.7;

    updateStatusBadge();
}

function saveSettings() {
    state.settings.apiKey = settingApiKey.value.trim();
    state.settings.model = settingModel.value;
    state.settings.temperature = parseFloat(settingTempSlider.value);

    localStorage.setItem("rust_agent_settings", JSON.stringify(state.settings));
    toggleModal(false);
    updateStatusBadge();
    showNotification("Pengaturan disimpan successfully!");
}

function resetSettings() {
    state.settings = {
        apiKey: "",
        model: "gemini-2.5-flash",
        temperature: 0.7
    };
    settingApiKey.value = "";
    settingModel.value = "gemini-2.5-flash";
    settingTempSlider.value = 0.7;
    settingTempVal.textContent = "0.7";

    localStorage.removeItem("rust_agent_settings");
    updateStatusBadge();
    showNotification("Pengaturan di-reset ke default.");
}

function toggleModal(show) {
    if (show) {
        settingsModal.classList.remove("hidden");
    } else {
        settingsModal.classList.add("hidden");
    }
}

function updateStatusBadge() {
    if (state.settings.apiKey) {
        statusBadge.className = "badge badge-connected";
        statusText.textContent = "Terkoneksi (API Key)";
    } else {
        statusBadge.className = "badge badge-demo";
        statusText.textContent = "Mode Demo";
    }
}

// Session Handlers
function loadSessions() {
    const saved = localStorage.getItem("rust_agent_sessions");
    if (saved) {
        state.sessions = JSON.parse(saved);
        if (state.sessions.length > 0) {
            state.activeSessionId = state.sessions[0].id;
        }
    }
    
    if (state.sessions.length === 0) {
        createNewSession();
    } else {
        renderChatList();
        renderActiveChat();
    }
}

function saveSessions() {
    localStorage.setItem("rust_agent_sessions", JSON.stringify(state.sessions));
}

function createNewSession() {
    const id = Date.now().toString();
    const newSession = {
        id,
        title: `Percakapan Baru`,
        messages: []
    };
    state.sessions.unshift(newSession);
    state.activeSessionId = id;
    
    saveSessions();
    renderChatList();
    renderActiveChat();
    
    chatInput.focus();
}

function selectSession(id) {
    state.activeSessionId = id;
    renderChatList();
    renderActiveChat();
}

function deleteActiveSession() {
    if (confirm("Apakah Anda yakin ingin menghapus chat saat ini?")) {
        state.sessions = state.sessions.filter(s => s.id !== state.activeSessionId);
        if (state.sessions.length > 0) {
            state.activeSessionId = state.sessions[0].id;
        } else {
            state.activeSessionId = null;
        }
        
        saveSessions();
        if (state.sessions.length === 0) {
            createNewSession();
        } else {
            renderChatList();
            renderActiveChat();
        }
    }
}

// Rendering UI
function renderChatList() {
    chatList.innerHTML = "";
    state.sessions.forEach(session => {
        const item = document.createElement("div");
        item.className = `chat-item ${session.id === state.activeSessionId ? 'active' : ''}`;
        item.addEventListener("click", () => selectSession(session.id));

        const title = document.createElement("span");
        title.className = "chat-item-title";
        title.textContent = session.title;

        const delBtn = document.createElement("button");
        delBtn.className = "chat-item-delete";
        delBtn.innerHTML = `&times;`;
        delBtn.addEventListener("click", (e) => {
            e.stopPropagation();
            deleteSession(session.id);
        });

        item.appendChild(title);
        item.appendChild(delBtn);
        chatList.appendChild(item);
    });
}

function deleteSession(id) {
    if (confirm("Hapus percakapan ini dari riwayat?")) {
        state.sessions = state.sessions.filter(s => s.id !== id);
        if (state.activeSessionId === id) {
            state.activeSessionId = state.sessions.length > 0 ? state.sessions[0].id : null;
        }
        saveSessions();
        if (state.sessions.length === 0) {
            createNewSession();
        } else {
            renderChatList();
            renderActiveChat();
        }
    }
}

function renderActiveChat() {
    const session = state.sessions.find(s => s.id === state.activeSessionId);
    if (!session) return;

    currentChatTitle.textContent = session.title;
    
    // Clear chat display
    chatMessages.innerHTML = "";
    
    if (session.messages.length === 0) {
        welcomeContainer.style.display = "flex";
        chatMessages.appendChild(welcomeContainer);
    } else {
        welcomeContainer.style.display = "none";
        
        // Render messages
        session.messages.forEach(msg => {
            if (msg.role === "user" || msg.role === "model") {
                // If it is a tool result or internal formatting, we don't display it directly as user bubble
                if (msg.content.startsWith("Tool result:")) return;
                if (msg.content.includes("<tool_call>") && msg.content.includes("</tool_call>")) return;

                renderMessageBubble(msg.role, msg.content, msg.steps);
            }
        });
    }
    scrollToBottom();
}

function renderMessageBubble(role, content, steps) {
    welcomeContainer.style.display = "none";
    
    const row = document.createElement("div");
    row.className = `message-row ${role}`;

    const bubble = document.createElement("div");
    bubble.className = "message-bubble";
    
    // Format markdown to HTML
    bubble.innerHTML = formatMarkdown(content);

    row.appendChild(bubble);
    chatMessages.appendChild(row);

    // If model response has reasoning steps, display them before the response bubble
    if (steps && steps.length > 0) {
        renderThinkingSteps(steps, row);
    }

    scrollToBottom();
}

function renderThinkingSteps(steps, messageRow) {
    const container = document.createElement("div");
    container.className = "thinking-container";
    
    const header = document.createElement("div");
    header.className = "thinking-header";
    
    const titleWrapper = document.createElement("div");
    titleWrapper.className = "thinking-title-wrapper";
    titleWrapper.innerHTML = `
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><path d="M12 6v6l4 2"></path></svg>
        <span>Proses Berpikir Agen (${steps.length} Langkah)</span>
    `;

    const chevron = document.createElement("div");
    chevron.className = "thinking-chevron";
    chevron.innerHTML = `
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
    `;

    header.appendChild(titleWrapper);
    header.appendChild(chevron);
    container.appendChild(header);

    const contentDiv = document.createElement("div");
    contentDiv.className = "thinking-content";

    const stepsList = document.createElement("div");
    stepsList.className = "thinking-steps-list";

    steps.forEach(step => {
        const stepItem = document.createElement("div");
        stepItem.className = `step-item ${step.status}`;

        const stepHeader = document.createElement("div");
        stepHeader.className = "step-header";
        stepHeader.innerHTML = `
            <span>${step.title}</span>
            <span class="step-status">${step.status === 'success' ? 'selesai' : step.status === 'pending' ? 'proses' : 'gagal'}</span>
        `;

        const stepLog = document.createElement("div");
        stepLog.className = "step-log";
        stepLog.textContent = step.log;

        stepItem.appendChild(stepHeader);
        stepItem.appendChild(stepLog);
        stepsList.appendChild(stepItem);
    });

    contentDiv.appendChild(stepsList);
    container.appendChild(contentDiv);

    // Collapsible Logic
    header.addEventListener("click", () => {
        container.classList.toggle("open");
    });

    // Insert reasoning container BEFORE the main response message row
    chatMessages.insertBefore(container, messageRow);
}

// Typing Indicator
function showTypingIndicator() {
    const row = document.createElement("div");
    row.className = "message-row model";
    row.id = "typing-indicator-row";

    const bubble = document.createElement("div");
    bubble.className = "typing-bubble";
    bubble.innerHTML = `
        <div class="typing-dot"></div>
        <div class="typing-dot"></div>
        <div class="typing-dot"></div>
    `;

    row.appendChild(bubble);
    chatMessages.appendChild(row);
    scrollToBottom();
}

function removeTypingIndicator() {
    const indicator = document.getElementById("typing-indicator-row");
    if (indicator) indicator.remove();
}

// Send Message Flow
async function sendMessage() {
    const text = chatInput.value.trim();
    if (!text) return;

    // Reset input
    chatInput.value = "";
    chatInput.style.height = "auto";

    const session = state.sessions.find(s => s.id === state.activeSessionId);
    if (!session) return;

    // Set title if it is the first message
    if (session.messages.length === 0) {
        session.title = text.length > 25 ? text.substring(0, 25) + "..." : text;
        renderChatList();
    }

    // Add User Message to State
    const userMsg = { role: "user", content: text };
    session.messages.push(userMsg);
    saveSessions();

    // Render User Message
    renderMessageBubble("user", text);

    // Show Loader
    showTypingIndicator();

    try {
        // Construct conversation payload
        // To save context size and respect backend expectations, map roles correctly
        const payloadMessages = session.messages.map(m => ({
            role: m.role,
            content: m.content
        }));

        const response = await fetch("/api/chat", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                messages: payloadMessages,
                api_key: state.settings.apiKey || null,
                model: state.settings.model,
                temperature: state.settings.temperature
            })
        });

        if (!response.ok) {
            const errData = await response.json();
            throw new Error(errData.error || "Gagal menghubungi agen.");
        }

        const data = await response.json(); // { steps: [], response: "" }

        // Remove Loader
        removeTypingIndicator();

        // Add Model Response to State
        const modelMsg = {
            role: "model",
            content: data.response,
            steps: data.steps
        };
        session.messages.push(modelMsg);
        saveSessions();

        // Render response
        renderMessageBubble("model", data.response, data.steps);

    } catch (error) {
        removeTypingIndicator();
        renderMessageBubble("model", `❌ **Error:** ${error.message}\n\n*Silakan cek koneksi internet Anda atau konfigurasikan ulang API Key Anda di menu Pengaturan.*`);
    }
}

// Utility Helpers
function scrollToBottom() {
    chatMessages.scrollTop = chatMessages.scrollHeight;
}

function showNotification(text) {
    const notification = document.createElement("div");
    notification.className = "animate-fade-in";
    notification.style.position = "fixed";
    notification.style.bottom = "2rem";
    notification.style.right = "2rem";
    notification.style.background = "rgba(16, 185, 129, 0.9)";
    notification.style.border = "1px solid rgba(255, 255, 255, 0.1)";
    notification.style.color = "white";
    notification.style.padding = "0.75rem 1.5rem";
    notification.style.borderRadius = "12px";
    notification.style.zIndex = "1000";
    notification.style.fontSize = "0.9rem";
    notification.style.boxShadow = "var(--shadow-premium)";
    notification.style.backdropFilter = "blur(10px)";
    
    document.body.appendChild(notification);
    notification.textContent = text;

    setTimeout(() => {
        notification.style.opacity = "0";
        notification.style.transition = "opacity 0.5s ease";
        setTimeout(() => notification.remove(), 500);
    }, 3000);
}

function updateUIState() {
    // Enable or disable clear chat button based on whether active session has messages
    const session = state.sessions.find(s => s.id === state.activeSessionId);
    if (session && session.messages.length > 0) {
        clearChatBtn.style.opacity = "1";
        clearChatBtn.style.pointerEvents = "auto";
    } else {
        clearChatBtn.style.opacity = "0.5";
        clearChatBtn.style.pointerEvents = "none";
    }
}

// Markdown Formatter Helper
function formatMarkdown(text) {
    if (!text) return "";
    
    // Escaping HTML characters to prevent XSS
    let escaped = text
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;");

    // Replace Markdown Code Blocks: ```lang code ```
    escaped = escaped.replace(/```(?:[a-zA-Z0-9]+)?\n([\s\S]*?)```/gim, (match, code) => {
        return `<pre><code>${code.trim()}</code></pre>`;
    });

    // Replace Inline Code: `code`
    escaped = escaped.replace(/`([^`]+)`/gim, "<code>$1</code>");

    // Replace bold markdown: **text**
    escaped = escaped.replace(/\*\*([^*]+)\*\*/gim, "<strong>$1</strong>");

    // Replace italic markdown: *text*
    escaped = escaped.replace(/\*([^*]+)\*/gim, "<em>$1</em>");

    // Replace headings: ###, ##, #
    escaped = escaped.replace(/^### (.*$)/gim, "<h3>$1</h3>");
    escaped = escaped.replace(/^## (.*$)/gim, "<h2>$1</h2>");
    escaped = escaped.replace(/^# (.*$)/gim, "<h1>$1</h1>");

    // Unordered lists (bullet points)
    escaped = escaped.replace(/^\s*[-*+]\s+(.*)$/gim, "<ul><li>$1</li></ul>");
    
    // Ordered lists
    escaped = escaped.replace(/^\s*(\d+)\.\s+(.*)$/gim, "<ol><li>$2</li></ol>");

    // Merge adjacent list elements
    escaped = escaped.replace(/<\/ul>\s*<ul>/gim, "");
    escaped = escaped.replace(/<\/ol>\s*<ol>/gim, "");

    // Line breaks (replace \n with <br>, except inside pre blocks)
    // We split by pre blocks to avoid adding <br> tags to code block formatting
    const parts = escaped.split(/(<pre>[\s\S]*?<\/pre>)/g);
    for (let i = 0; i < parts.length; i++) {
        if (!parts[i].startsWith("<pre>")) {
            parts[i] = parts[i].replace(/\n/g, "<br>");
        }
    }
    
    return parts.join("");
}
