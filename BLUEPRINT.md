# Project Blueprint

- **Version**: 1.1.0
- **Last Updated**: 2026-06-09
- **Description**: High-performance Agentic AI Chatbot with reasoning loops and local tool access built in Rust (Axum + Tokio) and Vanilla JS/CSS.

## Architecture & Modules

### 1. Frontend (Static Web UI)
- **Files**: `index.html`, `style.css`, `app.js`
- **Design System**: Sleek glassmorphism theme, dark mode, neon borders, and smooth transitions.
- **Features**: Collapsible thought/reasoning logs, settings modal for API key, model selection, and temperature. Intercepts backend intent URLs and opens them automatically in new browser tabs.

### 2. Backend (Rust Server)
- **Router**: Axum HTTP routes under `/api/chat` handling agent iterations.
- **Agent Loop (`src/agent.rs`)**:
  - Handles the reasoning loop with Gemini's system instructions.
  - Parsers for `<thought>` and `<tool_call>` tags.
- **Tools**:
  - `calculator`: Math parsing.
  - `get_system_info`: Server metrics and environment.
  - `get_time_date`: Time retrieval.
  - `list_directory`: Folder structure exploration.
  - `read_file`: Read workspace files.
  - `write_file`: Write/edit files in workspace.
  - `search_web`: Search the web using DuckDuckGo HTML parser.
  - `fetch_url`: Fetch web text content.
  - `post_to_threads`: Formats high-engagement Threads content and returns a Web Intent URL (`https://threads.net/intent/post?text=...`) to open it on the browser.
