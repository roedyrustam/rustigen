# Changelog

All notable changes to this project will be documented in this file.

## [1.3.4] - 2026-06-09

### Added
- **Automatic Browser-Fallback Posting to Threads**: Implemented Windows PowerShell `SendKeys` keyboard automation (`Ctrl+Enter`) inside the Rust backend. When the user publishes a post to Threads without an API token (or if the API fails), the backend opens the browser intent tab and automatically simulates the keyboard shortcut after 6 seconds to submit the post without manual clicking.
- **Access Token UI & State Management**: Integrated "Threads Access Token" password field in Settings modal, preserving the token in the browser's local storage and passing it dynamically in request payloads to `/api/chat`.

## [1.3.3] - 2026-06-09

### Added
- **YouTube Auto-Search & Discovery**: The YouTube Niche Analyzer now accepts general topic search queries (e.g. "Rust programming") instead of just handle strings, searching YouTube to auto-discover matching channels.
- **Search Metadata Banner**: Added a info banner at the start of reports showing the search keyword and the discovered target channel.
- **Demo Mode Search simulation**: Upgraded Demo Mode to parse topic/keyword search inputs and simulate discovery flow.

## [1.3.2] - 2026-06-09

### Added
- **Premium YouTube Profile Metadata**: Scrapes channel profile avatar image, subscriber count, total video count, and channel description to present a comprehensive, rich analytics card.
- **Markdown Image Rendering**: Added native parser support for standard Markdown images `![alt](url)` in the chat UI.
- **Neon Avatar Styling**: Styled channel logos with custom neon glow borders, floating text wrapping, and clean horizontal rule layouts.
- **Demo Mode simulation upgrade**: Upgraded YouTube analyzer simulation responses to include full metadata profile headers.

## [1.3.1] - 2026-06-09

### Added
- **YouTube Channel Niche Analyzer**: New backend tool `analyze_youtube_channel` that extracts channel IDs from handle pages, parses video view counts from RSS feeds, calculates channel view statistics, identifies outlier videos, and auto-generates micro niche suggestions.
- **YouTube Quick Action Chip**: Added `📺 Analisis YouTube` welcome chip to trigger the analyzer with a single click.

## [1.3.0] - 2026-06-09

### Added
- **Quick Action Chips (Prompt Templates)**: Clickable quick prompt chips under the input box for common scenarios (View Files, Search Web, Cargo Build, Math Calculation, draft Threads).
- **Chat Export (Markdown Download)**: Easily download the active conversation session as a nicely formatted Markdown `.md` file.
- **Conversation Search (Filter Sidebar)**: Case-insensitive search bar in the sidebar to dynamically filter conversation history based on titles and first messages.
- **Enhanced Markdown Parser**: Native formatting support inside chat bubbles for tables (with striped rows and header grids), blockquotes (premium glassmorphic layout), horizontal rules, and hyperlinks.
- **Keyboard Shortcuts Modal**: Keyboard shortcuts layout overlay with styling, accessible via `Ctrl+/` or `?`.
- **Global Hotkeys**: Navigation hotkeys including `Ctrl+N` (New Chat), `Ctrl+K` (Focus Search), `Ctrl+E` (Export Chat), `Ctrl+,` (Open Settings), and `Esc` (Close Modal).
- **Custom System Prompt (Personality)**: Editable custom system instruction textarea in settings to customize behavior, tone, or context parameters.

### Changed
- Settings payload expanded to include custom system instructions merged with default instructions.
- Version badge in sidebar header updated to `v1.3.0`.
- Delete chat and Export chat buttons state updated dynamically based on active conversation size.

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
