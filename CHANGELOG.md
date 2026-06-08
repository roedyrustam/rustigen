# Changelog

All notable changes to this project will be documented in this file.

## [1.2.0] - 2026-06-09

### Added
- **SSE Streaming Responses**: Agent responses now stream in real-time via Server-Sent Events, showing a live typing effect with a blinking cursor.
- **Execute Command Tool**: New `execute_command` tool allows the agent to run whitelisted shell commands (cargo, git, node, npm, python, etc.) with a 30-second timeout and output capture.
- **Context Window Management**: Configurable context window slider (5-50 messages) in settings to manage conversation history sent to the AI model.
- **Responsive Mobile Sidebar**: Hamburger menu button and slide-in sidebar with overlay for mobile devices (<768px).
- **Message Avatars**: Robot 🤖 and user 👤 avatar indicators beside chat bubbles.
- **Message Timestamps**: Each message now displays the time it was sent.
- **Code Language Labels**: Code blocks now show a language badge (e.g., "rust", "toml") in the top-left corner.
- **Animated Gradient Background**: Subtle, continuously moving radial gradient behind the main UI.
- **Streaming Cursor**: Blinking cursor animation during streaming text output.
- **Live Thinking Steps**: Reasoning steps appear in real-time during agent processing (auto-expanded).

### Changed
- Default model updated to `gemini-2.5-flash`.
- Backend endpoint `/api/chat` changed from JSON response to SSE stream.
- Agent loop refactored to use `mpsc` channels for streaming events.
- Message bubbles now slide in with smooth entrance animation.
- Version tag updated to v1.2.0.
- Scrollbar styling improved across all panels.
- Welcome features grid updated with Streaming and Command Execution highlights.

## [1.1.0] - 2026-06-09

### Added
- **Read File Tool**: Allows the agent to read contents of local workspace files.
- **Write File Tool**: Allows the agent to write or overwrite contents of files in the workspace.
- **Web Search Tool**: Integrated DuckDuckGo HTML scraping search allowing real-time web querying without external API keys.
- **Fetch URL Tool**: Allows scraping text content from public web page URLs.
- **Auto Post Threads Tool**: Generates high-engagement Threads content matching the timeline algorithm, and automatically opens a new browser tab using the Threads Web Intent (`https://www.threads.net/intent/post?text=...`) for publication (no API key/token required).
- Gemini Model selection in the settings modal updated to support Gemini 2.5 Flash, Gemini 2.5 Pro, and Gemini 2.0 Flash.

### Changed
- Default model updated to `gemini-2.5-flash`.
- Welcome features grid updated to showcase modern agentic abilities.
- Demo mode upgraded to asynchronously execute web searches, read project files, write files, fetch URLs, and open pre-filled Threads post composer tabs.
