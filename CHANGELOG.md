# Changelog

All notable changes to this project will be documented in this file.

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
