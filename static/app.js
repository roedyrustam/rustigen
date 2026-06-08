// State Management
let state = {
    sessions: [], // { id, title, messages: [] }
    activeSessionId: null,
    settings: {
        apiKey: "",
        model: "gemini-2.5-flash",
        temperature: 0.7,
        maxContext: 20,
        systemPrompt: ""
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

// Sidebar Elements (mobile)
const sidebar = document.getElementById("sidebar");
const sidebarOverlay = document.getElementById("sidebar-overlay");
const toggleSidebarBtn = document.getElementById("toggle-sidebar-btn");

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
const settingContextSlider = document.getElementById("setting-context-slider");
const settingContextVal = document.getElementById("setting-context-val");
const settingSystemPrompt = document.getElementById("setting-system-prompt");

// Quick Actions Chips
const quickActions = document.getElementById("quick-actions");

// Export button
const exportChatBtn = document.getElementById("export-chat-btn");

// Search inputs
const searchChats = document.getElementById("search-chats");
const clearSearchBtn = document.getElementById("clear-search-btn");

// Keyboard Shortcuts Modal Elements
const shortcutsModal = document.getElementById("shortcuts-modal");
const openShortcutsBtn = document.getElementById("open-shortcuts-btn");
const closeShortcutsBtn = document.getElementById("close-shortcuts-btn");
const closeShortcutsOkBtn = document.getElementById("close-shortcuts-ok-btn");

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

    // Mobile Sidebar Toggle
    toggleSidebarBtn.addEventListener("click", toggleSidebar);
    sidebarOverlay.addEventListener("click", closeSidebar);

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
    settingContextSlider.addEventListener("input", (e) => {
        settingContextVal.textContent = e.target.value;
    });

    // Quick Actions Click Handler
    if (quickActions) {
        quickActions.addEventListener("click", (e) => {
            const btn = e.target.closest(".quick-action-btn");
            if (btn) {
                const promptText = btn.dataset.prompt;
                if (promptText) {
                    chatInput.value = promptText;
                    chatInput.style.height = "auto";
                    chatInput.style.height = (chatInput.scrollHeight) + "px";
                    chatInput.focus();
                    sendMessage();
                }
            }
        });
    }

    // Export Chat Button
    if (exportChatBtn) {
        exportChatBtn.addEventListener("click", exportChat);
    }

    // Search chats in sidebar
    if (searchChats) {
        searchChats.addEventListener("input", (e) => {
            const query = e.target.value.toLowerCase();
            if (query) {
                clearSearchBtn.classList.remove("hidden");
            } else {
                clearSearchBtn.classList.add("hidden");
            }
            renderChatList();
        });
    }
    if (clearSearchBtn) {
        clearSearchBtn.addEventListener("click", () => {
            searchChats.value = "";
            clearSearchBtn.classList.add("hidden");
            renderChatList();
            searchChats.focus();
        });
    }

    // Shortcuts Modal Triggers
    if (openShortcutsBtn) {
        openShortcutsBtn.addEventListener("click", () => toggleShortcutsModal(true));
    }
    if (closeShortcutsBtn) {
        closeShortcutsBtn.addEventListener("click", () => toggleShortcutsModal(false));
    }
    if (closeShortcutsOkBtn) {
        closeShortcutsOkBtn.addEventListener("click", () => toggleShortcutsModal(false));
    }
    if (shortcutsModal) {
        shortcutsModal.addEventListener("click", (e) => {
            if (e.target === shortcutsModal) toggleShortcutsModal(false);
        });
    }

    // Global keyboard shortcuts
    document.addEventListener("keydown", (e) => {
        // Escape key to close modals
        if (e.key === "Escape") {
            toggleModal(false);
            toggleShortcutsModal(false);
            closeSidebar();
        }

        // Ctrl combinations
        if (e.ctrlKey) {
            switch (e.key.toLowerCase()) {
                case "n":
                    e.preventDefault();
                    createNewSession();
                    break;
                case "k":
                    e.preventDefault();
                    if (searchChats) {
                        searchChats.focus();
                        searchChats.select();
                    }
                    break;
                case "e":
                    e.preventDefault();
                    exportChat();
                    break;
                case ",":
                    e.preventDefault();
                    toggleModal(true);
                    break;
                case "/":
                    e.preventDefault();
                    toggleShortcutsModal(true);
                    break;
            }
        } else if (e.key === "?") {
            // Only open shortcuts if we're not inside input/textarea
            if (document.activeElement.tagName !== "INPUT" && document.activeElement.tagName !== "TEXTAREA") {
                e.preventDefault();
                toggleShortcutsModal(true);
            }
        }
    });
}

// --- SIDEBAR MOBILE ---
function toggleSidebar() {
    sidebar.classList.toggle("open");
    sidebarOverlay.classList.toggle("hidden");
}

function closeSidebar() {
    sidebar.classList.remove("open");
    sidebarOverlay.classList.add("hidden");
}

// --- SETTINGS ---
function loadSettings() {
    const saved = localStorage.getItem("rust_agent_settings");
    if (saved) {
        state.settings = { ...state.settings, ...JSON.parse(saved) };
    }
    
    // Populate form fields
    settingApiKey.value = state.settings.apiKey || "";
    settingModel.value = state.settings.model || "gemini-2.5-flash";
    settingTempSlider.value = state.settings.temperature || 0.7;
    settingTempVal.textContent = state.settings.temperature || 0.7;
    settingContextSlider.value = state.settings.maxContext || 20;
    settingContextVal.textContent = state.settings.maxContext || 20;
    settingSystemPrompt.value = state.settings.systemPrompt || "";

    updateStatusBadge();
}

function saveSettings() {
    state.settings.apiKey = settingApiKey.value.trim();
    state.settings.model = settingModel.value;
    state.settings.temperature = parseFloat(settingTempSlider.value);
    state.settings.maxContext = parseInt(settingContextSlider.value);
    state.settings.systemPrompt = settingSystemPrompt.value.trim();

    localStorage.setItem("rust_agent_settings", JSON.stringify(state.settings));
    toggleModal(false);
    updateStatusBadge();
    showNotification("✅ Pengaturan berhasil disimpan!");
}

function resetSettings() {
    state.settings = {
        apiKey: "",
        model: "gemini-2.5-flash",
        temperature: 0.7,
        maxContext: 20,
        systemPrompt: ""
    };
    settingApiKey.value = "";
    settingModel.value = "gemini-2.5-flash";
    settingTempSlider.value = 0.7;
    settingTempVal.textContent = "0.7";
    settingContextSlider.value = 20;
    settingContextVal.textContent = "20";
    settingSystemPrompt.value = "";

    localStorage.removeItem("rust_agent_settings");
    updateStatusBadge();
    showNotification("🔄 Pengaturan di-reset ke default.");
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

// --- SESSION HANDLERS ---
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
    closeSidebar();
    
    chatInput.focus();
}

function selectSession(id) {
    state.activeSessionId = id;
    renderChatList();
    renderActiveChat();
    closeSidebar();
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

// --- RENDERING ---
function renderChatList() {
    chatList.innerHTML = "";
    const query = searchChats ? searchChats.value.toLowerCase().trim() : "";
    
    const filteredSessions = state.sessions.filter(session => {
        if (!query) return true;
        if (session.title.toLowerCase().includes(query)) return true;
        const firstUserMsg = session.messages.find(m => m.role === "user");
        if (firstUserMsg && firstUserMsg.content.toLowerCase().includes(query)) return true;
        return false;
    });

    filteredSessions.forEach(session => {
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

    if (filteredSessions.length === 0 && query) {
        const noResults = document.createElement("div");
        noResults.className = "help-text";
        noResults.style.textAlign = "center";
        noResults.style.padding = "1.5rem 1rem";
        noResults.textContent = "Tidak ada percakapan ditemukan.";
        chatList.appendChild(noResults);
    }
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
        if (quickActions) quickActions.classList.remove("hidden");
    } else {
        welcomeContainer.style.display = "none";
        if (quickActions) quickActions.classList.add("hidden");
        
        // Render messages
        session.messages.forEach(msg => {
            if (msg.role === "user" || msg.role === "model") {
                // If it is a tool result or internal formatting, we don't display it directly
                if (msg.content.startsWith("Tool result:")) return;
                if (msg.content.includes("<tool_call>") && msg.content.includes("</tool_call>")) return;

                renderMessageBubble(msg.role, msg.content, msg.steps, msg.timestamp);
            }
        });
    }
    updateUIState();
    scrollToBottom();
}

function getTimestamp() {
    const now = new Date();
    return now.toLocaleTimeString("id-ID", { hour: "2-digit", minute: "2-digit" });
}

function renderMessageBubble(role, content, steps, timestamp) {
    welcomeContainer.style.display = "none";
    
    const row = document.createElement("div");
    row.className = `message-row ${role}`;

    // Avatar
    const avatar = document.createElement("div");
    avatar.className = "message-avatar";
    avatar.textContent = role === "model" ? "🤖" : "👤";

    // Content wrapper
    const contentWrapper = document.createElement("div");
    contentWrapper.className = "message-content-wrapper";

    const bubble = document.createElement("div");
    bubble.className = "message-bubble";
    
    // Format markdown to HTML
    bubble.innerHTML = formatMarkdown(content);

    // Add Copy buttons and language labels to code blocks
    addCodeEnhancements(bubble);

    contentWrapper.appendChild(bubble);

    // Timestamp
    if (timestamp) {
        const ts = document.createElement("div");
        ts.className = "message-timestamp";
        ts.textContent = timestamp;
        contentWrapper.appendChild(ts);
    }

    row.appendChild(avatar);
    row.appendChild(contentWrapper);
    chatMessages.appendChild(row);

    // If model response has reasoning steps, display them before the response bubble
    if (steps && steps.length > 0) {
        renderThinkingSteps(steps, row);
    }

    scrollToBottom();
    return { row, bubble, contentWrapper };
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

    const avatar = document.createElement("div");
    avatar.className = "message-avatar";
    avatar.textContent = "🤖";

    const bubble = document.createElement("div");
    bubble.className = "typing-bubble";
    bubble.innerHTML = `
        <div class="typing-dot"></div>
        <div class="typing-dot"></div>
        <div class="typing-dot"></div>
    `;

    row.appendChild(avatar);
    row.appendChild(bubble);
    chatMessages.appendChild(row);
    scrollToBottom();
}

function removeTypingIndicator() {
    const indicator = document.getElementById("typing-indicator-row");
    if (indicator) indicator.remove();
}

// --- SEND MESSAGE WITH SSE STREAMING ---
let activeAbortController = null;

async function sendMessage() {
    const text = chatInput.value.trim();
    if (!text) return;

    // Hide quick action chips once message is sent
    if (quickActions) {
        quickActions.classList.add("hidden");
    }

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
    const timestamp = getTimestamp();
    const userMsg = { role: "user", content: text, timestamp };
    session.messages.push(userMsg);
    saveSessions();

    // Render User Message
    renderMessageBubble("user", text, null, timestamp);

    // Show Loader
    showTypingIndicator();

    // Abort previous stream if any
    if (activeAbortController) {
        activeAbortController.abort();
    }
    activeAbortController = new AbortController();

    try {
        // Construct conversation payload
        const payloadMessages = session.messages
            .filter(m => m.role === "user" || m.role === "model")
            .map(m => ({ role: m.role, content: m.content }));

        const response = await fetch("/api/chat", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                messages: payloadMessages,
                api_key: state.settings.apiKey || null,
                model: state.settings.model,
                temperature: state.settings.temperature,
                max_context: state.settings.maxContext,
                system_prompt: state.settings.systemPrompt || null
            }),
            signal: activeAbortController.signal
        });

        if (!response.ok) {
            const errText = await response.text();
            throw new Error(errText || "Gagal menghubungi agen.");
        }

        // Remove typing indicator
        removeTypingIndicator();

        // Prepare streaming UI elements
        const modelTimestamp = getTimestamp();
        let steps = [];
        let responseText = "";
        let thinkingContainer = null;
        let responseBubble = null;
        let responseRow = null;
        let responseWrapper = null;
        let streamingCursor = null;

        // Read SSE stream
        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";

        while (true) {
            const { done, value } = await reader.read();
            if (done) break;

            buffer += decoder.decode(value, { stream: true });

            // Parse SSE events from buffer
            const lines = buffer.split("\n");
            buffer = lines.pop(); // Keep incomplete last line in buffer

            for (const line of lines) {
                if (line.startsWith("data:")) {
                    const jsonStr = line.slice(5).trim();
                    if (!jsonStr) continue;

                    try {
                        const event = JSON.parse(jsonStr);
                        
                        switch (event.type) {
                            case "step": {
                                const step = event.step;
                                steps.push(step);

                                // Create or update the thinking container
                                if (!thinkingContainer) {
                                    thinkingContainer = createLiveThinkingContainer();
                                    chatMessages.appendChild(thinkingContainer);
                                }
                                addStepToThinkingContainer(thinkingContainer, step, steps.length);
                                scrollToBottom();
                                break;
                            }

                            case "chunk": {
                                // Create response bubble on first chunk
                                if (!responseBubble) {
                                    welcomeContainer.style.display = "none";
                                    responseRow = document.createElement("div");
                                    responseRow.className = "message-row model";

                                    const avatar = document.createElement("div");
                                    avatar.className = "message-avatar";
                                    avatar.textContent = "🤖";

                                    responseWrapper = document.createElement("div");
                                    responseWrapper.className = "message-content-wrapper";

                                    responseBubble = document.createElement("div");
                                    responseBubble.className = "message-bubble";

                                    streamingCursor = document.createElement("span");
                                    streamingCursor.className = "streaming-cursor";

                                    responseBubble.appendChild(streamingCursor);
                                    responseWrapper.appendChild(responseBubble);

                                    responseRow.appendChild(avatar);
                                    responseRow.appendChild(responseWrapper);
                                    chatMessages.appendChild(responseRow);
                                }

                                responseText += event.text;
                                
                                // Remove cursor, re-render content, add cursor back
                                if (streamingCursor && streamingCursor.parentNode) {
                                    streamingCursor.remove();
                                }
                                responseBubble.innerHTML = formatMarkdown(responseText);
                                responseBubble.appendChild(streamingCursor);
                                scrollToBottom();
                                break;
                            }

                            case "open_url": {
                                window.open(event.url, '_blank');
                                break;
                            }

                            case "error": {
                                removeTypingIndicator();
                                renderMessageBubble("model", `❌ **Error:** ${event.message}\n\n*Silakan cek koneksi internet Anda atau konfigurasikan ulang API Key Anda di menu Pengaturan.*`, null, getTimestamp());
                                break;
                            }

                            case "done": {
                                // Finalize: remove streaming cursor
                                if (streamingCursor && streamingCursor.parentNode) {
                                    streamingCursor.remove();
                                }
                                // Re-render final content with code enhancements
                                if (responseBubble) {
                                    responseBubble.innerHTML = formatMarkdown(responseText);
                                    addCodeEnhancements(responseBubble);

                                    // Add timestamp
                                    const ts = document.createElement("div");
                                    ts.className = "message-timestamp";
                                    ts.textContent = modelTimestamp;
                                    responseWrapper.appendChild(ts);
                                }
                                break;
                            }
                        }
                    } catch (parseErr) {
                        console.warn("Failed to parse SSE event:", jsonStr, parseErr);
                    }
                }
            }
        }

        // Save model response to state
        const modelMsg = {
            role: "model",
            content: responseText,
            steps: steps,
            timestamp: modelTimestamp
        };
        session.messages.push(modelMsg);
        saveSessions();

    } catch (error) {
        if (error.name === 'AbortError') return;
        removeTypingIndicator();
        renderMessageBubble("model", `❌ **Error:** ${error.message}\n\n*Silakan cek koneksi internet Anda atau konfigurasikan ulang API Key Anda di menu Pengaturan.*`, null, getTimestamp());
    } finally {
        activeAbortController = null;
    }
}

// --- LIVE THINKING CONTAINER ---
function createLiveThinkingContainer() {
    const container = document.createElement("div");
    container.className = "thinking-container open"; // Start open for live streaming

    const header = document.createElement("div");
    header.className = "thinking-header";

    const titleWrapper = document.createElement("div");
    titleWrapper.className = "thinking-title-wrapper";
    titleWrapper.dataset.stepCount = "0";
    titleWrapper.innerHTML = `
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><path d="M12 6v6l4 2"></path></svg>
        <span>Proses Berpikir Agen...</span>
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

    contentDiv.appendChild(stepsList);
    container.appendChild(contentDiv);

    // Collapsible
    header.addEventListener("click", () => {
        container.classList.toggle("open");
    });

    return container;
}

function addStepToThinkingContainer(container, step, count) {
    const titleSpan = container.querySelector(".thinking-title-wrapper span");
    if (titleSpan) {
        titleSpan.textContent = `Proses Berpikir Agen (${count} Langkah)`;
    }

    const stepsList = container.querySelector(".thinking-steps-list");
    
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
}

// --- UTILITY HELPERS ---
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
    // Enable or disable clear chat button and export button based on whether active session has messages
    const session = state.sessions.find(s => s.id === state.activeSessionId);
    const hasMessages = session && session.messages.length > 0;
    
    if (clearChatBtn) {
        clearChatBtn.style.opacity = hasMessages ? "1" : "0.5";
        clearChatBtn.style.pointerEvents = hasMessages ? "auto" : "none";
    }
    if (exportChatBtn) {
        exportChatBtn.style.opacity = hasMessages ? "1" : "0.5";
        exportChatBtn.style.pointerEvents = hasMessages ? "auto" : "none";
    }
}

// --- MARKDOWN FORMATTER ---
function formatMarkdown(text) {
    if (!text) return "";
    
    // Escaping HTML characters to prevent XSS
    let escaped = text
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;");

    // Replace Tables (do this before other replacements)
    const lines = escaped.split("\n");
    let inTable = false;
    let tableRows = [];
    let processedLines = [];

    for (let line of lines) {
        const trimmed = line.trim();
        const isRow = trimmed.startsWith("|") && trimmed.endsWith("|");
        
        if (isRow) {
            if (!inTable) {
                inTable = true;
                tableRows = [];
            }
            const cells = trimmed
                .slice(1, -1)
                .split("|")
                .map(c => c.trim());
            tableRows.push(cells);
        } else {
            if (inTable) {
                processedLines.push(renderHtmlTable(tableRows));
                inTable = false;
            }
            processedLines.push(line);
        }
    }
    if (inTable) {
        processedLines.push(renderHtmlTable(tableRows));
    }
    escaped = processedLines.join("\n");

    // Replace Markdown Code Blocks: ```lang code ```
    escaped = escaped.replace(/```([a-zA-Z0-9]*)\n([\s\S]*?)```/gim, (match, lang, code) => {
        const langLabel = lang ? `<span class="code-lang-label">${lang}</span>` : '';
        return `<pre>${langLabel}<code>${code.trim()}</code></pre>`;
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

    // Blockquotes: > text (note: > is escaped to &gt;)
    escaped = escaped.replace(/^\s*&gt;\s+(.*)$/gim, "<blockquote>$1</blockquote>");
    escaped = escaped.replace(/<\/blockquote>\s*<blockquote>/gim, "<br>");

    // Horizontal rules: --- or ***
    escaped = escaped.replace(/^\s*(?:---|\*\*\*)\s*$/gim, "<hr>");

    // Unordered lists (bullet points)
    escaped = escaped.replace(/^\s*[-*+]\s+(.*)$/gim, "<ul><li>$1</li></ul>");
    
    // Ordered lists
    escaped = escaped.replace(/^\s*(\d+)\.\s+(.*)$/gim, "<ol><li>$2</li></ol>");

    // Merge adjacent list elements
    escaped = escaped.replace(/<\/ul>\s*<ul>/gim, "");
    escaped = escaped.replace(/<\/ol>\s*<ol>/gim, "");

    // Images: ![alt](url)
    escaped = escaped.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, '<img src="$2" alt="$1" class="markdown-image">');

    // Links: [text](url)
    escaped = escaped.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank">$1</a>');

    // Line breaks (replace \n with <br>, except inside pre blocks)
    const parts = escaped.split(/(<pre>[\s\S]*?<\/pre>)/g);
    for (let i = 0; i < parts.length; i++) {
        if (!parts[i].startsWith("<pre>")) {
            parts[i] = parts[i].replace(/\n/g, "<br>");
        }
    }
    
    return parts.join("");
}

function addCodeEnhancements(container) {
    const preBlocks = container.querySelectorAll("pre");
    preBlocks.forEach((pre) => {
        pre.style.position = "relative";
        
        // Copy button
        const copyBtn = document.createElement("button");
        copyBtn.className = "copy-code-btn";
        copyBtn.innerHTML = `
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
            <span>Salin</span>
        `;
        
        copyBtn.addEventListener("click", async () => {
            const code = pre.querySelector("code");
            const text = code ? code.innerText : pre.innerText;
            try {
                await navigator.clipboard.writeText(text);
                copyBtn.classList.add("copied");
                copyBtn.innerHTML = `
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
                    <span>Tersalin!</span>
                `;
                setTimeout(() => {
                    copyBtn.classList.remove("copied");
                    copyBtn.innerHTML = `
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                        <span>Salin</span>
                    `;
                }, 2000);
            } catch (err) {
                console.error("Gagal menyalin teks: ", err);
            }
        });
        
        pre.appendChild(copyBtn);
    });
}

// --- v1.3.0 NEW HELPER FUNCTIONS ---
function renderHtmlTable(rows) {
    if (rows.length === 0) return "";
    
    // Check if row 1 is delimiter row (like |---|---| or | :--- |)
    const hasHeaders = rows.length > 1 && rows[1].every(cell => {
        return cell.match(/^[:\-\s]+$/) !== null;
    });
    
    let html = "<table>";
    let startIdx = 0;
    
    if (hasHeaders) {
        html += "<thead><tr>";
        rows[0].forEach(h => {
            html += `<th>${h}</th>`;
        });
        html += "</tr></thead>";
        startIdx = 2; // skip headers and separator line
    }
    
    html += "<tbody>";
    for (let i = startIdx; i < rows.length; i++) {
        if (i === 1 && rows[i].every(cell => cell.match(/^[:\-\s]+$/) !== null)) {
            continue;
        }
        html += "<tr>";
        rows[i].forEach(cell => {
            html += `<td>${cell}</td>`;
        });
        html += "</tr>";
    }
    html += "</tbody></table>";
    return html;
}

function toggleShortcutsModal(show) {
    if (show) {
        shortcutsModal.classList.remove("hidden");
        if (closeShortcutsOkBtn) closeShortcutsOkBtn.focus();
    } else {
        shortcutsModal.classList.add("hidden");
    }
}

function exportChat() {
    const session = state.sessions.find(s => s.id === state.activeSessionId);
    if (!session || session.messages.length === 0) {
        showNotification("⚠️ Tidak ada pesan untuk diekspor!");
        return;
    }

    let markdown = `# ${session.title}\n`;
    markdown += `*Tanggal Ekspor: ${new Date().toLocaleDateString("id-ID")} ${new Date().toLocaleTimeString("id-ID")}*\n\n---\n\n`;

    session.messages.forEach(msg => {
        if (msg.role === "user" || msg.role === "model") {
            if (msg.content.startsWith("Tool result:")) return;
            if (msg.content.includes("<tool_call>") && msg.content.includes("</tool_call>")) return;

            const roleName = msg.role === "user" ? "User" : "RustAgent 🤖";
            markdown += `### **${roleName}** _(${msg.timestamp || ''})_\n\n${msg.content}\n\n---\n\n`;
        }
    });

    const blob = new Blob([markdown], { type: "text/markdown;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    
    const cleanTitle = session.title.replace(/[^a-z0-9]/gi, '_').toLowerCase();
    link.href = url;
    link.setAttribute("download", `chat_${cleanTitle || 'export'}.md`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
    
    showNotification("📥 Chat berhasil diekspor sebagai Markdown!");
}
