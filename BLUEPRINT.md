# Project Blueprint

- **Version**: 1.3.4
- **Last Updated**: 2026-06-09
- **Description**: High-performance Agentic AI Chatbot with reasoning loops, SSE streaming, local tool access, custom personality settings, hotkeys, search, Markdown export, Markdown image rendering, and premium YouTube niche analysis (with auto-search channel discovery, avatar, description, and sub count) built in Rust (Axum + Tokio) and Vanilla JS/CSS.

## Architecture & Modules

### 1. Frontend (Static Web UI)
- **Files**: `index.html`, `style.css`, `app.js`
- **Design System**: Sleek glassmorphism theme, dark mode, neon borders, animated gradient background, and smooth transitions.
- **Features**: 
  - SSE streaming with real-time typing effect and blinking cursor.
  - Collapsible live thought/reasoning logs, message avatars, and timestamps.
  - Code language labels and copy code buttons.
  - Settings modal for API key, model selection, temperature, context window, and **Custom System Prompt (Personality)**.
  - **Quick Action Chips**: Prompt templates under input for common scenarios (View Files, Search Web, Build, Math, Threads, **YouTube Niche Analysis**).
  - **Chat Export**: Easily export the current chat as a Markdown `.md` file download.
  - **Conversation Search**: Filter chat history in the sidebar dynamically.
  - **Enhanced Markdown**: Support for tables, blockquotes, horizontal rules, hyperlinks, and image rendering `![alt](url)` inside chat bubbles.
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
  - `post_to_threads`: Auto-publishes posts directly via Threads Graph API if an Access Token is configured in Settings. Falls back to opening a browser pre-filled composer tab and running a PowerShell SendKeys simulation to automatically trigger the `Ctrl+Enter` publish hotkey after 6 seconds.
  - `analyze_youtube_channel`: Searches YouTube for keywords to auto-discover matching channels, scrapes channel HTML profile page (resolving ID, avatar, subscribers, videos, description), parses RSS XML feed views/dates/titles, detects outlier high-performing topics, and suggests micro niches.
