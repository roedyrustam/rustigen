use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Instant;
use anyhow::Result;
use tracing::info;
use tokio::sync::mpsc;

static START_TIME: OnceLock<Instant> = OnceLock::new();

// Ensure START_TIME is initialized when the server starts
pub fn init_uptime() {
    START_TIME.get_or_init(Instant::now);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String, // "user" or "model"
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_context: Option<usize>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub title: String,
    pub log: String,
    pub status: String, // "pending", "success", "error"
}

/// SSE event types sent to the frontend
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum SseEvent {
    #[serde(rename = "step")]
    Step { step: AgentStep },
    #[serde(rename = "chunk")]
    Chunk { text: String },
    #[serde(rename = "open_url")]
    OpenUrl { url: String },
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Deserialize)]
struct ToolCall {
    tool: String,
    expression: Option<String>,
    path: Option<String>,
    content: Option<String>,
    query: Option<String>,
    url: Option<String>,
    text: Option<String>,
    command: Option<String>,
    args: Option<Vec<String>>,
}

// --- MATH EVALUATOR ---
fn tokenize(expr: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c.is_digit(10) || c == '.' {
            let mut num = String::new();
            while let Some(&nc) = chars.peek() {
                if nc.is_digit(10) || nc == '.' {
                    num.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            tokens.push(num);
        } else if "+-*/()".contains(c) {
            tokens.push(c.to_string());
            chars.next();
        } else {
            return Err(format!("Invalid character: {}", c));
        }
    }
    Ok(tokens)
}

fn parse_expr(tokens: &[String], index: &mut usize) -> Result<f64, String> {
    let mut value = parse_term(tokens, index)?;
    while *index < tokens.len() {
        let op = &tokens[*index];
        if op == "+" || op == "-" {
            *index += 1;
            let next_value = parse_term(tokens, index)?;
            if op == "+" {
                value += next_value;
            } else {
                value -= next_value;
            }
        } else {
            break;
        }
    }
    Ok(value)
}

fn parse_term(tokens: &[String], index: &mut usize) -> Result<f64, String> {
    let mut value = parse_factor(tokens, index)?;
    while *index < tokens.len() {
        let op = &tokens[*index];
        if op == "*" || op == "/" {
            *index += 1;
            let next_value = parse_factor(tokens, index)?;
            if op == "*" {
                value *= next_value;
            } else {
                if next_value == 0.0 {
                    return Err("Division by zero".to_string());
                }
                value /= next_value;
            }
        } else {
            break;
        }
    }
    Ok(value)
}

fn parse_factor(tokens: &[String], index: &mut usize) -> Result<f64, String> {
    if *index >= tokens.len() {
        return Err("Unexpected end of expression".to_string());
    }
    let token = &tokens[*index];
    if token == "(" {
        *index += 1;
        let value = parse_expr(tokens, index)?;
        if *index >= tokens.len() || tokens[*index] != ")" {
            return Err("Expected matching ')'".to_string());
        }
        *index += 1;
        Ok(value)
    } else if let Ok(val) = token.parse::<f64>() {
        *index += 1;
        Ok(val)
    } else if token == "-" {
        *index += 1;
        let value = parse_factor(tokens, index)?;
        Ok(-value)
    } else {
        Err(format!("Unexpected token: {}", token))
    }
}

pub fn evaluate_math(expr: &str) -> Result<f64, String> {
    let tokens = tokenize(expr)?;
    let mut index = 0;
    let val = parse_expr(&tokens, &mut index)?;
    if index < tokens.len() {
        return Err("Trailing characters in expression".to_string());
    }
    Ok(val)
}

// --- TOOLS IMPLEMENTATION ---
fn run_calculator(expr: &str) -> String {
    match evaluate_math(expr) {
        Ok(res) => format!("Result of expression `{}` is: {}", expr, res),
        Err(e) => format!("Error calculating `{}`: {}", expr, e),
    }
}

fn run_system_info() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let current_dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "Unknown".to_string());
    let uptime = START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0);
    let hours = uptime / 3600;
    let minutes = (uptime % 3600) / 60;
    let seconds = uptime % 60;
    
    format!(
        "System Information:\n\
         - Operating System: {}\n\
         - Architecture: {}\n\
         - Current Working Directory: {}\n\
         - Server Uptime: {}h {}m {}s",
        os, arch, current_dir, hours, minutes, seconds
    )
}

fn run_get_time_date() -> String {
    let now = chrono::Local::now();
    format!("Current Local Time: {}", now.format("%Y-%m-%d %H:%M:%S %Z"))
}

fn run_list_directory(path_opt: Option<&str>) -> String {
    let base_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let target_path = if let Some(p) = path_opt {
        // Prevent path traversal outside workspace for security
        let path = std::path::Path::new(p);
        if path.is_absolute() || path.components().any(|c| c == std::path::Component::ParentDir) {
            return "Error: Path must be relative and remain inside the workspace".to_string();
        }
        base_path.join(path)
    } else {
        base_path.clone()
    };

    match std::fs::read_dir(&target_path) {
        Ok(entries) => {
            let mut result = format!("Contents of directory '{}':\n", target_path.to_string_lossy());
            let mut files = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    let file_type = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        "Dir "
                    } else {
                        "File"
                    };
                    files.push(format!("  [{}] {}", file_type, name));
                }
            }
            if files.is_empty() {
                result.push_str("  (empty directory)");
            } else {
                result.push_str(&files.join("\n"));
            }
            result
        }
        Err(e) => format!("Error reading directory: {}", e),
    }
}

fn run_read_file(path_str: &str) -> String {
    let base_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let path = std::path::Path::new(path_str);
    if path.is_absolute() || path.components().any(|c| c == std::path::Component::ParentDir) {
        return "Error: Path must be relative and remain inside the workspace".to_string();
    }
    let target_path = base_path.join(path);
    match std::fs::read_to_string(&target_path) {
        Ok(content) => content,
        Err(e) => format!("Error reading file: {}", e),
    }
}

fn run_write_file(path_str: &str, content: &str) -> String {
    let base_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let path = std::path::Path::new(path_str);
    if path.is_absolute() || path.components().any(|c| c == std::path::Component::ParentDir) {
        return "Error: Path must be relative and remain inside the workspace".to_string();
    }
    let target_path = base_path.join(path);
    if let Some(parent) = target_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return format!("Error creating parent directories: {}", e);
        }
    }
    match std::fs::write(&target_path, content) {
        Ok(_) => format!("Successfully wrote content to file: {}", path_str),
        Err(e) => format!("Error writing file: {}", e),
    }
}

async fn run_fetch_url(client: &reqwest::Client, url: &str) -> String {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return "Error: URL must start with http:// or https://".to_string();
    }
    match client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36")
        .send()
        .await
    {
        Ok(res) => {
            let status = res.status();
            if !status.is_success() {
                return format!("Error fetching URL: Server returned status {}", status);
            }
            match res.text().await {
                Ok(html) => {
                    let stripped = strip_html_tags(&html);
                    let mut lines = Vec::new();
                    for line in stripped.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            lines.push(trimmed);
                        }
                    }
                    let cleaned = lines.join("\n");
                    if cleaned.len() > 5000 {
                        format!("(Showing first 5000 characters of page content)\n\n{}", &cleaned[..5000])
                    } else {
                        cleaned
                    }
                }
                Err(e) => format!("Error reading response text: {}", e),
            }
        }
        Err(e) => format!("Error sending request to URL: {}", e),
    }
}

async fn run_search_web(client: &reqwest::Client, query: &str) -> String {
    info!("Searching web for: {}", query);
    match client
        .get("https://html.duckduckgo.com/html/")
        .query(&[("q", query)])
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36")
        .send()
        .await
    {
        Ok(res) => {
            let status = res.status();
            if !status.is_success() {
                return format!("Error performing web search: Server returned status {}", status);
            }
            match res.text().await {
                Ok(html) => {
                    let results = parse_ddg_html(&html);
                    if results.is_empty() {
                        "No search results found.".to_string()
                    } else {
                        let mut output = format!("Search results for '{}':\n\n", query);
                        for (i, (title, url, snippet)) in results.into_iter().enumerate() {
                            output.push_str(&format!("{}. **{}**\n   URL: {}\n   Snippet: {}\n\n", i + 1, title, url, snippet));
                        }
                        output
                    }
                }
                Err(e) => format!("Error reading search results: {}", e),
            }
        }
        Err(e) => format!("Error sending search request: {}", e),
    }
}

// --- EXECUTE COMMAND TOOL ---
const COMMAND_WHITELIST: &[&str] = &[
    "cargo", "rustc", "rustup", "dir", "ls", "type", "cat", "echo",
    "node", "npm", "npx", "git", "python", "python3", "pip",
    "whoami", "hostname", "ping", "curl", "where", "which",
];

async fn run_execute_command(command: &str, args: &[String]) -> String {
    // Validate command against whitelist
    let cmd_base = command.split(['/', '\\']).last().unwrap_or(command);
    let cmd_lower = cmd_base.to_lowercase();
    let cmd_clean = cmd_lower.strip_suffix(".exe").unwrap_or(&cmd_lower);

    if !COMMAND_WHITELIST.iter().any(|&w| w == cmd_clean) {
        return format!(
            "Error: Command '{}' is not in the allowed whitelist.\nAllowed commands: {}",
            command,
            COMMAND_WHITELIST.join(", ")
        );
    }

    let workspace = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    info!("Executing command: {} {:?} in {:?}", command, args, workspace);

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        tokio::process::Command::new(command)
            .args(args)
            .current_dir(&workspace)
            .output()
    ).await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(-1);

            let mut result_str = format!("Exit Code: {}\n", exit_code);

            if !stdout.is_empty() {
                let stdout_trimmed = if stdout.len() > 10000 {
                    format!("{}...\n(output truncated at 10000 chars)", &stdout[..10000])
                } else {
                    stdout.to_string()
                };
                result_str.push_str(&format!("\n--- STDOUT ---\n{}", stdout_trimmed));
            }

            if !stderr.is_empty() {
                let stderr_trimmed = if stderr.len() > 5000 {
                    format!("{}...\n(stderr truncated at 5000 chars)", &stderr[..5000])
                } else {
                    stderr.to_string()
                };
                result_str.push_str(&format!("\n--- STDERR ---\n{}", stderr_trimmed));
            }

            if stdout.is_empty() && stderr.is_empty() {
                result_str.push_str("\n(No output produced)");
            }

            result_str
        }
        Ok(Err(e)) => format!("Error executing command '{}': {}", command, e),
        Err(_) => format!("Error: Command '{}' timed out after 30 seconds", command),
    }
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for c in input.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            output.push(c);
        }
    }
    output = output
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'");
    output.trim().to_string()
}

fn parse_ddg_html(html: &str) -> Vec<(String, String, String)> {
    let mut results = Vec::new();
    let mut search_idx = 0;
    while let Some(start_pos) = html[search_idx..].find("class=\"result__a\"") {
        let absolute_start = search_idx + start_pos;
        let href_start = match html[..absolute_start].rfind("href=\"") {
            Some(idx) => idx + 6,
            None => {
                search_idx = absolute_start + 16;
                continue;
            }
        };
        let href_end = match html[href_start..].find("\"") {
            Some(idx) => href_start + idx,
            None => {
                search_idx = absolute_start + 16;
                continue;
            }
        };
        let url = &html[href_start..href_end];
        let decoded_url = decode_ddg_url(url);

        let title_content_start = match html[absolute_start..].find(">") {
            Some(idx) => absolute_start + idx + 1,
            None => {
                search_idx = absolute_start + 16;
                continue;
            }
        };
        let title_content_end = match html[title_content_start..].find("</a>") {
            Some(idx) => title_content_start + idx,
            None => {
                search_idx = absolute_start + 16;
                continue;
            }
        };
        let raw_title = &html[title_content_start..title_content_end];
        let title = strip_html_tags(raw_title);

        let snippet_search_pos = title_content_end;
        let mut snippet = String::new();
        if let Some(snippet_class_pos) = html[snippet_search_pos..].find("class=\"result__snippet\"") {
            let snip_start = snippet_search_pos + snippet_class_pos;
            let next_result_pos = html[title_content_end..].find("class=\"result__a\"");
            if next_result_pos.is_none() || snippet_class_pos < next_result_pos.unwrap() {
                if let Some(snip_content_start_offset) = html[snip_start..].find(">") {
                    let snip_content_start = snip_start + snip_content_start_offset + 1;
                    if let Some(snip_content_end_offset) = html[snip_content_start..].find("</a>") {
                        let snip_content_end = snip_content_start + snip_content_end_offset;
                        let raw_snip = &html[snip_content_start..snip_content_end];
                        snippet = strip_html_tags(raw_snip);
                    }
                }
            }
        }
        
        results.push((title, decoded_url, snippet));
        search_idx = title_content_end;
        if results.len() >= 5 {
            break;
        }
    }
    results
}

fn decode_ddg_url(url: &str) -> String {
    if let Some(pos) = url.find("uddg=") {
        let encoded_part = &url[pos + 5..];
        let end_pos = encoded_part.find('&').unwrap_or(encoded_part.len());
        let part = &encoded_part[..end_pos];
        percent_decode(part)
    } else if url.starts_with("//") {
        format!("https:{}", url)
    } else {
        url.to_string()
    }
}

fn percent_decode(s: &str) -> String {
    let mut res = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next();
            let h2 = chars.next();
            if let (Some(c1), Some(c2)) = (h1, h2) {
                let hex_str = format!("{}{}", c1, c2);
                if let Ok(val) = u8::from_str_radix(&hex_str, 16) {
                    res.push(val as char);
                } else {
                    res.push('%');
                    res.push(c1);
                    res.push(c2);
                }
            } else {
                res.push('%');
                if let Some(c1) = h1 { res.push(c1); }
            }
        } else if c == '+' {
            res.push(' ');
        } else {
            res.push(c);
        }
    }
    res
}

fn url_encode(s: &str) -> String {
    let mut res = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                res.push(b as char);
            }
            b' ' => {
                res.push_str("%20");
            }
            _ => {
                res.push_str(&format!("%{:02X}", b));
            }
        }
    }
    res
}

pub struct ToolResult {
    pub output: String,
    pub open_url: Option<String>,
}

impl ToolResult {
    fn new(output: String) -> Self {
        Self { output, open_url: None }
    }
    fn with_url(output: String, url: String) -> Self {
        Self { output, open_url: Some(url) }
    }
}

// --- CONTEXT WINDOW MANAGEMENT ---
fn trim_conversation(messages: &[Message], max_messages: usize) -> Vec<Message> {
    if messages.len() <= max_messages {
        return messages.to_vec();
    }
    // Always keep the first message (initial user query for context) + last N messages
    let mut trimmed = Vec::with_capacity(max_messages + 1);
    trimmed.push(messages[0].clone());
    let start = messages.len().saturating_sub(max_messages);
    for msg in &messages[start..] {
        trimmed.push(msg.clone());
    }
    trimmed
}

// --- AGENT LOOP EXECUTION ---

async fn execute_tool(
    tool_name: &str,
    call: &ToolCall,
    client: &reqwest::Client,
) -> ToolResult {
    match tool_name {
        "calculator" => {
            let res = if let Some(ref expr) = call.expression {
                run_calculator(expr)
            } else {
                "Error: Missing 'expression' argument for calculator".to_string()
            };
            ToolResult::new(res)
        }
        "get_system_info" => ToolResult::new(run_system_info()),
        "get_time_date" => ToolResult::new(run_get_time_date()),
        "list_directory" => ToolResult::new(run_list_directory(call.path.as_deref())),
        "read_file" => {
            let res = if let Some(path) = &call.path {
                run_read_file(path)
            } else {
                "Error: Missing 'path' argument for read_file".to_string()
            };
            ToolResult::new(res)
        }
        "write_file" => {
            let res = if let (Some(path), Some(content)) = (&call.path, &call.content) {
                run_write_file(path, content)
            } else {
                "Error: Missing 'path' or 'content' argument for write_file".to_string()
            };
            ToolResult::new(res)
        }
        "search_web" => {
            let res = if let Some(query) = &call.query {
                run_search_web(client, query).await
            } else {
                "Error: Missing 'query' argument for search_web".to_string()
            };
            ToolResult::new(res)
        }
        "fetch_url" => {
            let res = if let Some(url) = &call.url {
                run_fetch_url(client, url).await
            } else {
                "Error: Missing 'url' argument for fetch_url".to_string()
            };
            ToolResult::new(res)
        }
        "execute_command" => {
            let res = if let Some(command) = &call.command {
                let args = call.args.clone().unwrap_or_default();
                run_execute_command(command, &args).await
            } else {
                "Error: Missing 'command' argument for execute_command".to_string()
            };
            ToolResult::new(res)
        }
        "post_to_threads" => {
            if let Some(text) = &call.text {
                let encoded = url_encode(text);
                let intent_url = format!("https://www.threads.net/intent/post?text={}", encoded);
                let output = format!(
                    "Successfully generated Threads post. Opening browser tab to publish...\n\nContent:\n\"{}\"",
                    text
                );
                ToolResult::with_url(output, intent_url)
            } else {
                ToolResult::new("Error: Missing 'text' argument for post_to_threads".to_string())
            }
        }
        _ => ToolResult::new(format!("Error: Unknown tool '{}'", tool_name)),
    }
}

/// Streaming agent loop — sends SSE events through the channel as it processes
pub async fn run_agent_loop_streaming(
    req: ChatRequest,
    client: &reqwest::Client,
    env_api_key: Option<String>,
    tx: mpsc::Sender<SseEvent>,
) {
    let api_key = req.api_key.or(env_api_key);
    let model = req.model.unwrap_or_else(|| "gemini-2.5-flash".to_string());
    let temp = req.temperature.unwrap_or(0.7);
    let max_ctx = req.max_context.unwrap_or(20);

    // If no API key is provided, execute Demo Mode
    if api_key.is_none() {
        run_demo_mode_streaming(&req.messages, client, &tx).await;
        let _ = tx.send(SseEvent::Done).await;
        return;
    }

    let key = api_key.unwrap();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, key
    );

    let mut system_instruction = "\
You are an advanced agentic coding and analysis assistant. You have access to the following tools:
- `calculator(expression)`: Solve mathematical equations. Returns the evaluated float or error. Example request: <tool_call>{\"tool\": \"calculator\", \"expression\": \"(12 + 8) * 5\"}</tool_call>
- `get_system_info()`: Retrieve OS, Architecture, Current Directory, and Uptime metrics. Example request: <tool_call>{\"tool\": \"get_system_info\"}</tool_call>
- `get_time_date()`: Get the current local date, time, and timezone. Example request: <tool_call>{\"tool\": \"get_time_date\"}</tool_call>
- `list_directory(path)`: List file names in a directory relative to workspace. Optional path argument. Example request: <tool_call>{\"tool\": \"list_directory\", \"path\": \"src\"}</tool_call>
- `read_file(path)`: Read the entire text content of a file relative to workspace. Example request: <tool_call>{\"tool\": \"read_file\", \"path\": \"src/main.rs\"}</tool_call>
- `write_file(path, content)`: Write or overwrite text content of a file relative to workspace. Example request: <tool_call>{\"tool\": \"write_file\", \"path\": \"src/temp.txt\", \"content\": \"Hello World!\"}</tool_call>
- `search_web(query)`: Search the web using DuckDuckGo search. Returns matching titles, URLs, and snippets. Example request: <tool_call>{\"tool\": \"search_web\", \"query\": \"Rust Axum routing tutorial\"}</tool_call>
- `fetch_url(url)`: Fetch and extract plain text content from a web page URL. Example request: <tool_call>{\"tool\": \"fetch_url\", \"url\": \"https://example.com\"}</tool_call>
- `execute_command(command, args)`: Execute a whitelisted shell command in the workspace. Returns stdout, stderr, and exit code. The command has a 30-second timeout. Allowed commands include: cargo, rustc, git, node, npm, python, dir, ls, echo, etc. Example request: <tool_call>{\"tool\": \"execute_command\", \"command\": \"cargo\", \"args\": [\"build\"]}</tool_call>
- `post_to_threads(text)`: Auto-publish a post to Threads. Generates high-engagement Threads content matching the timeline algorithm (strong hook, short paragraphs, spacing, ending question to drive replies, under 500 characters, no external links). Example request: <tool_call>{\"tool\": \"post_to_threads\", \"text\": \"Did you know that Rust Axum compiles down to a single binary under 10MB? 🦀\\n\\nHere is why it is the future of web dev...\\n\\nWhat is your favorite Rust framework?\"}</tool_call>

To use a tool, you must respond with a JSON object enclosed in `<tool_call>` and `</tool_call>` tags.
Before making a tool call, or if you do not need a tool call, write out your detailed thinking process enclosed in `<thought>` and `</thought>` tags. Always output the tool_call tags on a separate line.
When creating content for Threads, always craft it to optimize timeline algorithm engagement: a compelling hook, 500 chars limit, emojis, neat paragraph spacing, and an end prompt/question to invite replies.

For example:
<thought>
I need to check the current directory contents to see the main source files. Let's run the list_directory tool.
</thought>
<tool_call>
{\"tool\": \"list_directory\"}
</tool_call>

Once you receive the tool result, analyze it, make further tool calls if needed, and when finished, output your final response outside of any `<thought>` or `<tool_call>` tags. Always explain the results clearly.".to_string();

    if let Some(ref custom_prompt) = req.system_prompt {
        let trimmed = custom_prompt.trim();
        if !trimmed.is_empty() {
            system_instruction = format!("{}\n\n{}", trimmed, system_instruction);
        }
    }

    let mut conversation = req.messages.clone();
    let mut loop_count = 0;
    const MAX_LOOPS: usize = 5;

    while loop_count < MAX_LOOPS {
        loop_count += 1;
        info!("Running agent loop iteration {}", loop_count);

        // Trim conversation to max context window
        let trimmed = trim_conversation(&conversation, max_ctx);

        // Map conversation into Gemini API format
        let mut contents = Vec::new();
        for msg in &trimmed {
            contents.push(serde_json::json!({
                "role": msg.role,
                "parts": [{ "text": msg.content }]
            }));
        }

        let request_body = serde_json::json!({
            "contents": contents,
            "systemInstruction": {
                "parts": [{ "text": system_instruction }]
            },
            "generationConfig": {
                "temperature": temp
            }
        });

        let response = match client.post(&url).json(&request_body).send().await {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(SseEvent::Error {
                    message: format!("Failed to send request to Gemini API: {}", e),
                }).await;
                let _ = tx.send(SseEvent::Done).await;
                return;
            }
        };

        let status = response.status();
        if !status.is_success() {
            let err_text = response.text().await.unwrap_or_default();
            let _ = tx.send(SseEvent::Error {
                message: format!("Gemini API returned status {}: {}", status, err_text),
            }).await;
            let _ = tx.send(SseEvent::Done).await;
            return;
        }

        let resp_json: serde_json::Value = match response.json().await {
            Ok(j) => j,
            Err(e) => {
                let _ = tx.send(SseEvent::Error {
                    message: format!("Failed to parse Gemini API response: {}", e),
                }).await;
                let _ = tx.send(SseEvent::Done).await;
                return;
            }
        };

        // Extract output text
        let output_text = match resp_json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
            Some(t) => t.to_string(),
            None => {
                let _ = tx.send(SseEvent::Error {
                    message: format!("Empty response from Gemini API: {:?}", resp_json),
                }).await;
                let _ = tx.send(SseEvent::Done).await;
                return;
            }
        };

        info!("Agent output: {}", output_text);

        // Parse thought if present
        if let Some(start_thought) = output_text.find("<thought>") {
            if let Some(end_thought) = output_text.find("</thought>") {
                let thought_content = output_text[start_thought + 9..end_thought].trim().to_string();
                let _ = tx.send(SseEvent::Step {
                    step: AgentStep {
                        title: "Thinking Process".to_string(),
                        log: thought_content,
                        status: "success".to_string(),
                    }
                }).await;
            }
        }

        // Parse tool call if present
        let mut has_tool_call = false;
        if let Some(start_tool) = output_text.find("<tool_call>") {
            if let Some(end_tool) = output_text.find("</tool_call>") {
                let tool_json_str = &output_text[start_tool + 11..end_tool];
                if let Ok(tool_call) = serde_json::from_str::<ToolCall>(tool_json_str) {
                    has_tool_call = true;
                    
                    let tool_name = tool_call.tool.clone();
                    
                    // Send pending step
                    let _ = tx.send(SseEvent::Step {
                        step: AgentStep {
                            title: format!("Executing Tool: {}", tool_name),
                            log: format!("Arguments: {}\nRunning...", tool_json_str.trim()),
                            status: "pending".to_string(),
                        }
                    }).await;
                    
                    // Execute tool
                    let tool_result = execute_tool(&tool_name, &tool_call, client).await;

                    if let Some(ref url) = tool_result.open_url {
                        let _ = tx.send(SseEvent::OpenUrl { url: url.clone() }).await;
                    }

                    let tool_result_str = tool_result.output;
                    
                    // Send completed step
                    let _ = tx.send(SseEvent::Step {
                        step: AgentStep {
                            title: format!("Executing Tool: {}", tool_name),
                            log: format!("Arguments: {}\n\nResult:\n{}", tool_json_str.trim(), tool_result_str),
                            status: "success".to_string(),
                        }
                    }).await;

                    // Append model's thought & tool call to history
                    conversation.push(Message {
                        role: "model".to_string(),
                        content: output_text.clone(),
                    });

                    // Append tool result as user input to history
                    conversation.push(Message {
                        role: "user".to_string(),
                        content: format!("Tool result: {}", tool_result_str),
                    });
                }
            }
        }

        if !has_tool_call {
            // No tool call, means the agent finished and returned its final response.
            let clean_response = output_text
                .replace("<thought>", "")
                .replace("</thought>", "")
                .split("</thought>")
                .last()
                .unwrap_or(&output_text)
                .trim()
                .to_string();
            
            // Stream response in chunks for typing effect
            let chars: Vec<char> = clean_response.chars().collect();
            let chunk_size = 12; // characters per chunk for smooth typing
            for chunk in chars.chunks(chunk_size) {
                let text: String = chunk.iter().collect();
                let _ = tx.send(SseEvent::Chunk { text }).await;
                tokio::time::sleep(std::time::Duration::from_millis(15)).await;
            }
            break;
        }
    }

    if loop_count >= MAX_LOOPS {
        let _ = tx.send(SseEvent::Chunk {
            text: "Agent loop exceeded maximum turns without final response.".to_string(),
        }).await;
    }

    let _ = tx.send(SseEvent::Done).await;
}

// --- DEMO MODE STREAMING ---
async fn run_demo_mode_streaming(
    messages: &[Message],
    client: &reqwest::Client,
    tx: &mpsc::Sender<SseEvent>,
) {
    let last_user_msg = messages
        .iter()
        .filter(|m| m.role == "user")
        .last()
        .map(|m| m.content.to_lowercase())
        .unwrap_or_default();

    let response: String;

    if last_user_msg.contains("sistem") || last_user_msg.contains("system") || last_user_msg.contains("info") {
        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: "User is asking for system information. I will call the `get_system_info()` tool to retrieve operating system metrics, working directory, and uptime.".to_string(),
                status: "success".to_string(),
            }
        }).await;
        
        let sys_info = run_system_info();
        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Executing Tool: get_system_info".to_string(),
                log: format!("Arguments: {{}}\n\nResult:\n{}", sys_info),
                status: "success".to_string(),
            }
        }).await;

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: "I have retrieved the system information. I will now present it clearly to the user, highlighting the operating system, current directory, and server uptime.".to_string(),
                status: "success".to_string(),
            }
        }).await;

        response = format!(
            "Berikut adalah informasi sistem dari server agen Rust Anda (Mode Demo):\n\n\
            * **OS**: `{}`\n\
            * **Arsitektur**: `{}`\n\
            * **Direktori Kerja**: `{}`\n\
            * **Uptime Server**: `{}`\n\n\
            *Catatan: Anda berada di Mode Demo karena tidak ada API Key yang dipasang. Pasang API Key di Pengaturan untuk terhubung dengan Gemini API sesungguhnya!*",
            std::env::consts::OS,
            std::env::consts::ARCH,
            std::env::current_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default(),
            {
                let uptime = START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0);
                format!("{} jam {} menit {} detik", uptime / 3600, (uptime % 3600) / 60, uptime % 60)
            }
        );
    } else if last_user_msg.contains("hitung") || last_user_msg.contains("math") || last_user_msg.contains("calculator") || last_user_msg.contains("+") || last_user_msg.contains("*") {
        let mut expr = "15 * (24 + 6)".to_string();
        for word in last_user_msg.split_whitespace() {
            if word.chars().any(|c| c.is_digit(10)) && word.chars().any(|c| "+-*/()".contains(c)) {
                expr = word.to_string();
                break;
            }
        }

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: format!("User requested a math calculation. I will parse the expression `{}` and execute the `calculator` tool to solve it.", expr),
                status: "success".to_string(),
            }
        }).await;

        let calc_res = run_calculator(&expr);
        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Executing Tool: calculator".to_string(),
                log: format!("Arguments: {{\n  \"expression\": \"{}\"\n}}\n\nResult:\n{}", expr, calc_res),
                status: "success".to_string(),
            }
        }).await;

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: "Calculation completed successfully. I will explain the solution steps and provide the final result to the user.".to_string(),
                status: "success".to_string(),
            }
        }).await;

        response = format!(
            "Hasil perhitungan matematika Anda (Mode Demo):\n\n\
            Ekspresi: `{}`\n\
            **Hasil**: `{}`\n\n\
            *Pasang Gemini API Key di menu Pengaturan (ikon gir di pojok kiri bawah) untuk menghubungkan agen ini dengan model LLM sesungguhnya!*",
            expr,
            evaluate_math(&expr).map(|v| v.to_string()).unwrap_or_else(|e| format!("Error ({})", e))
        );
    } else if last_user_msg.contains("baca file") || last_user_msg.contains("read file") || last_user_msg.contains("buka file") {
        let mut path_found = None;
        for word in last_user_msg.split_whitespace() {
            if word.contains('.') || word.contains('/') || word.contains('\\') {
                path_found = Some(word.trim_matches(|c| c == '`' || c == '\'' || c == '"').to_string());
                break;
            }
        }
        let path = path_found.unwrap_or_else(|| "Cargo.toml".to_string());
        
        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: format!("User requested to read the file `{}`. I will execute the `read_file` tool to retrieve its content.", path),
                status: "success".to_string(),
            }
        }).await;
        
        let content = run_read_file(&path);
        
        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Executing Tool: read_file".to_string(),
                log: format!("Arguments: {{\n  \"path\": \"{}\"\n}}\n\nResult:\n(Successfully read file content)", path),
                status: "success".to_string(),
            }
        }).await;

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: "Content of file has been read. I will now display it inside a formatted markdown code block for the user.".to_string(),
                status: "success".to_string(),
            }
        }).await;

        response = format!(
            "Berikut adalah isi dari file `{}` (Mode Demo):\n\n\
            ```rust\n\
            {}\n\
            ```\n\n\
            *Catatan: Anda sedang di Mode Demo. Masukkan Gemini API Key untuk otonomi penuh!*",
            path, content
        );
    } else if last_user_msg.contains("tulis file") || last_user_msg.contains("buat file") || last_user_msg.contains("write file") {
        let mut path_found = None;
        for word in last_user_msg.split_whitespace() {
            if word.contains('.') || word.contains('/') || word.contains('\\') {
                path_found = Some(word.trim_matches(|c| c == '`' || c == '\'' || c == '"').to_string());
                break;
            }
        }
        let path = path_found.unwrap_or_else(|| "test_demo.txt".to_string());
        let mock_content = format!("Hello from Rust Agentic Chatbot Demo Mode!\nCreated at: {}", run_get_time_date());

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: format!("User requested to write a file at `{}`. I will run `write_file` to save the content.", path),
                status: "success".to_string(),
            }
        }).await;

        let write_res = run_write_file(&path, &mock_content);

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Executing Tool: write_file".to_string(),
                log: format!("Arguments: {{\n  \"path\": \"{}\",\n  \"content\": \"{}\"\n}}\n\nResult:\n{}", path, mock_content, write_res),
                status: "success".to_string(),
            }
        }).await;

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: "File has been written. I will report the success to the user.".to_string(),
                status: "success".to_string(),
            }
        }).await;

        response = format!(
            "Berhasil menulis file di `{}` (Mode Demo):\n\n\
            **Status**: `{}`\n\n\
            *Isi yang ditulis:*\n\
            ```text\n\
            {}\n\
            ```",
            path, write_res, mock_content
        );
    } else if last_user_msg.contains("jalankan") || last_user_msg.contains("execute") || last_user_msg.contains("run ") || last_user_msg.contains("command") {
        let command = "cargo";
        let args = vec!["--version".to_string()];

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: format!("User wants to execute a command. I will run `execute_command` with `{} {}` to demonstrate shell execution capabilities.", command, args.join(" ")),
                status: "success".to_string(),
            }
        }).await;

        let cmd_res = run_execute_command(command, &args).await;

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Executing Tool: execute_command".to_string(),
                log: format!("Arguments: {{\n  \"command\": \"{}\",\n  \"args\": [\"{}\"]\n}}\n\nResult:\n{}", command, args.join("\", \""), cmd_res),
                status: "success".to_string(),
            }
        }).await;

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: "Command executed successfully. I will present the output to the user.".to_string(),
                status: "success".to_string(),
            }
        }).await;

        response = format!(
            "Perintah `{} {}` berhasil dijalankan (Mode Demo):\n\n\
            ```\n\
            {}\n\
            ```\n\n\
            **Perintah yang diizinkan**: `{}`\n\n\
            *Catatan: Tool ini memiliki whitelist keamanan dan timeout 30 detik.*",
            command, args.join(" "), cmd_res, COMMAND_WHITELIST.join("`, `")
        );
    } else if last_user_msg.contains("cari ") || last_user_msg.contains("search ") || last_user_msg.contains("google ") || last_user_msg.contains("browsing ") {
        let query = if let Some(idx) = last_user_msg.find("cari ") {
            &messages.iter().filter(|m| m.role == "user").last().unwrap().content[idx + 5..]
        } else if let Some(idx) = last_user_msg.find("search ") {
            &messages.iter().filter(|m| m.role == "user").last().unwrap().content[idx + 7..]
        } else if let Some(idx) = last_user_msg.find("google ") {
            &messages.iter().filter(|m| m.role == "user").last().unwrap().content[idx + 7..]
        } else if let Some(idx) = last_user_msg.find("browsing ") {
            &messages.iter().filter(|m| m.role == "user").last().unwrap().content[idx + 9..]
        } else {
            "Rust Axum framework"
        };
        
        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: format!("User wants to search the web for `{}`. I will call `search_web` to get matching web pages.", query),
                status: "success".to_string(),
            }
        }).await;

        let search_res = run_search_web(client, query).await;

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Executing Tool: search_web".to_string(),
                log: format!("Arguments: {{\n  \"query\": \"{}\"\n}}\n\nResult:\n(Fetched search results)", query),
                status: "success".to_string(),
            }
        }).await;

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: "Web search results retrieved. I will format them nicely in markdown for presentation.".to_string(),
                status: "success".to_string(),
            }
        }).await;

        response = format!(
            "Berikut adalah hasil pencarian web ril untuk kata kunci `{}` (Mode Demo):\n\n\
            {}\n\n\
            *Catatan: Agen ini menggunakan DuckDuckGo secara otonom untuk pencarian web!*",
            query, search_res
        );
    } else if last_user_msg.contains("http://") || last_user_msg.contains("https://") || last_user_msg.contains("buka url") || last_user_msg.contains("fetch url") {
        let mut url_found = "https://example.com".to_string();
        for word in last_user_msg.split_whitespace() {
            if word.starts_with("http://") || word.starts_with("https://") {
                url_found = word.trim_matches(|c| c == '`' || c == '\'' || c == '"' || c == '<' || c == '>').to_string();
                break;
            }
        }

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: format!("User provided a URL: `{}`. I will execute the `fetch_url` tool to scrape and read the web page text.", url_found),
                status: "success".to_string(),
            }
        }).await;

        let fetch_res = run_fetch_url(client, &url_found).await;

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Executing Tool: fetch_url".to_string(),
                log: format!("Arguments: {{\n  \"url\": \"{}\"\n}}\n\nResult:\n(Successfully fetched page content)", url_found),
                status: "success".to_string(),
            }
        }).await;

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: "Web page content has been fetched. I will now present a summary or the first few paragraphs to the user.".to_string(),
                status: "success".to_string(),
            }
        }).await;

        response = format!(
            "Berikut adalah konten teks dari URL `{}` (Mode Demo):\n\n\
            ```text\n\
            {}\n\
            ```",
            url_found, fetch_res
        );
    } else if last_user_msg.contains("file") || last_user_msg.contains("direktori") || last_user_msg.contains("ls") || last_user_msg.contains("list") {
        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: "User wants to see the files inside the directory. I will call `list_directory()` for the current directory root.".to_string(),
                status: "success".to_string(),
            }
        }).await;

        let dir_res = run_list_directory(None);
        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Executing Tool: list_directory".to_string(),
                log: format!("Arguments: {{}}\n\nResult:\n{}", dir_res),
                status: "success".to_string(),
            }
        }).await;

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: "Directory contents retrieved. I will format the directory contents list neatly in markdown code block for readability.".to_string(),
                status: "success".to_string(),
            }
        }).await;

        response = format!(
            "Berikut adalah daftar file di direktori kerja server agen (Mode Demo):\n\n\
            ```text\n\
            {}\n\
            ```\n\n\
            *Masukkan Gemini API Key untuk menguji agen otonom sesungguhnya yang dapat menelusuri file dan memecahkan kode Anda secara dinamis!*",
            dir_res
        );
    } else if last_user_msg.contains("threads") || last_user_msg.contains("posting") || last_user_msg.contains("post ") {
        let post_text = if last_user_msg.contains("posting ") {
            let user_content = messages.iter().filter(|m| m.role == "user").last().unwrap().content.clone();
            let idx = user_content.to_lowercase().find("posting ").unwrap();
            user_content[idx + 8..].trim().to_string()
        } else {
            "Selamat pagi developer! 🦀\n\nSedang mengembangkan agen AI otonom dengan Rust hari ini. Kecepatannya luar biasa!\n\nBagaimana stack andalan kalian tahun ini? Let's discuss! 👇".to_string()
        };

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: "User wants to post to Threads. I will analyze the content and optimize it for the Threads timeline algorithm (hook, length, emojis, and ending question).".to_string(),
                status: "success".to_string(),
            }
        }).await;

        let encoded = url_encode(&post_text);
        let intent_url = format!("https://www.threads.net/intent/post?text={}", encoded);

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Executing Tool: post_to_threads".to_string(),
                log: format!("Arguments: {{\n  \"text\": \"{}\"\n}}\n\nResult:\nSuccessfully generated Threads Intent URL. Opening browser tab...", post_text),
                status: "success".to_string(),
            }
        }).await;

        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: "Threads post intent URL generated. Sending command to open a new tab on the user's browser.".to_string(),
                status: "success".to_string(),
            }
        }).await;

        let _ = tx.send(SseEvent::OpenUrl { url: intent_url }).await;

        response = format!(
            "🚀 **Membuka postingan di Threads...** (Mode Demo)\n\n\
            **Konten Terposting:**\n\
            ```text\n\
            {}\n\
            ```\n\n\
            **Analisis Algoritma Timeline Threads:**\n\
            1. **Hook Awal**: 1-2 baris pertama menarik perhatian pembaca.\n\
            2. **Call to Action (CTA)**: Pertanyaan di akhir mendorong interaksi (kolom balasan).\n\
            3. **Panjang Karakter**: {} karakter (sangat di bawah limit 500).\n\n\
            *Tab baru Threads web composer seharusnya terbuka otomatis secara lokal. Klik 'Post' untuk mempublikasikan!*",
            post_text, post_text.chars().count()
        );
    } else {
        let _ = tx.send(SseEvent::Step {
            step: AgentStep {
                title: "Thinking Process".to_string(),
                log: "User greeted me or asked a general question. Since I am in Demo Mode, I will introduce my capabilities and explain how the user can activate the full Agent features with their Gemini API key.".to_string(),
                status: "success".to_string(),
            }
        }).await;

        response = format!(
            "Halo! Saya adalah **Rust Agentic Chatbot** 🤖. Saat ini saya berjalan dalam **Mode Demo** karena Anda belum memasang API Key.\n\n\
            **Kemampuan Agen Saya:**\n\
            1. **Berpikir Multi-langkah**: Saya menganalisis masalah Anda terlebih dahulu dan membuat rencana aksi.\n\
            2. **Akses Alat Sistem (Tools) Lengkap**:\n\
               * 🧮 **Kalkulator**: Untuk perhitungan matematis.\n\
               * 🖥️ **Informasi Sistem**: Mengakses spesifikasi sistem backend.\n\
               * 📂 **Daftar Direktori**: Membaca struktur file otonom.\n\
               * 📄 **Baca File**: Membuka isi file proyek.\n\
               * ✏️ **Tulis File**: Menulis atau mengedit file proyek.\n\
               * 🌐 **Pencarian Web**: Mencari informasi di internet secara langsung.\n\
               * 🔗 **Scrape URL**: Mengambil data teks dari halaman web.\n\
               * ⚡ **Eksekusi Perintah**: Menjalankan perintah shell (cargo, git, dll).\n\n\
            **Cara Mengaktifkan Fitur Penuh:**\n\
            * Klik ikon **Pengaturan (Gir)** ⚙️ di pojok kiri bawah UI.\n\
            * Masukkan **Gemini API Key** Anda.\n\
            * Pilih model (seperti `gemini-2.5-flash` atau `gemini-2.5-pro`).\n\
            * Mulai ajukan pertanyaan kompleks dan perhatikan saya bekerja otonom menggunakan perkakas saya!"
        );
    }

    // Stream the response in chunks for typing effect
    let chars: Vec<char> = response.chars().collect();
    let chunk_size = 12;
    for chunk in chars.chunks(chunk_size) {
        let text: String = chunk.iter().collect();
        let _ = tx.send(SseEvent::Chunk { text }).await;
        tokio::time::sleep(std::time::Duration::from_millis(15)).await;
    }
}
