# Project Blueprint

- **Version**: 1.3.0
- **Last Updated**: 2026-06-09
- **Description**: High-performance Agentic AI Chatbot with reasoning loops, SSE streaming, local tool access, custom personality settings, hotkeys, search, and Markdown export built in Rust (Axum + Tokio) and Vanilla JS/CSS.

## Architecture & Modules

### 1. Frontend (Static Web UI)
- **Files**: `index.html`, `style.css`, `app.js`
- **Design System**: Sleek glassmorphism theme, dark mode, neon borders, animated gradient background, and smooth transitions.
- **Features**: 
  - SSE streaming with real-time typing effect and blinking cursor.
  - Collapsible live thought/reasoning logs, message avatars, and timestamps.
  - Code language labels and copy code buttons.
  - Settings modal for API key, model selection, temperature, context window, and **Custom System Prompt (Personality)**.
  - **Quick Action Chips**: Prompt templates under input for common scenarios (View Files, Search Web, Build, Math, Threads).
  - **Chat Export**: Easily export the current chat as a Markdown `.md` file download.
  - **Conversation Search**: Filter chat history in the sidebar dynamically.
  - **Enhanced Markdown**: Support for tables, blockquotes, horizontal rules, and hyperlinks inside chat bubbles.
  - **Keyboard Shortcuts**: Built-in hotkeys modal (Ctrl+N, Ctrl+K, Ctrl+E, Ctrl+/, Ctrl+, Esc) and support for `?` key.
  - Responsive mobile sidebar with hamburger menu.

### 2. Backend (Rust Server)
- **Router**: Axum HTTP routes under `/api/chat` serving SSE (Server-Sent Events) stream.
- **Agent Loop (`src/agent.rs`)**:
  - Handles the reasoning loop with Gemini's system instructions.
  - Dynamic Custom System Prompt: Merges custom user personality settings with the default system instructions.
  - Parsers for `<thought>` and `<tool_call>` tags.
  - Streams events via `tokio::sync::mpsc` channel: `step`, `chunk`, `open_url`, `done`, `error`.
  - Context window trimming to manage conversation length.
- **Tools**:
  - `calculator`: Math parsing.
  - `get_system_info`: Server metrics and environment.
  - `get_time_date`: Time retrieval.
  - `list_directory`: Folder structure exploration.
  - `read_file`: Read workspace files.
  - `write_file`: Write/edit files in workspace.
  - `search_web`: Search the web using DuckDuckGo HTML parser.
  - `fetch_url`: Fetch web text content.
  - `execute_command`: Run whitelisted shell commands with 30s timeout (cargo, git, node, npm, python, etc.).
  - `post_to_threads`: Formats high-engagement Threads content and returns a Web Intent URL (`https://threads.net/intent/post?text=...`) to open it on the browser.
