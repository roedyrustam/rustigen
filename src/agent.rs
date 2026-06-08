use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Instant;
use anyhow::{Result, Context};
use tracing::info;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub title: String,
    pub log: String,
    pub status: String, // "pending", "success", "error"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub steps: Vec<AgentStep>,
    pub response: String,
}

#[derive(Debug, Deserialize)]
struct ToolCall {
    tool: String,
    expression: Option<String>,
    path: Option<String>,
    content: Option<String>,
    query: Option<String>,
    url: Option<String>,
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

// --- AGENT LOOP EXECUTION ---

async fn execute_tool(tool_name: &str, call: &ToolCall, client: &reqwest::Client) -> String {
    match tool_name {
        "calculator" => {
            if let Some(ref expr) = call.expression {
                run_calculator(expr)
            } else {
                "Error: Missing 'expression' argument for calculator".to_string()
            }
        }
        "get_system_info" => run_system_info(),
        "get_time_date" => run_get_time_date(),
        "list_directory" => run_list_directory(call.path.as_deref()),
        "read_file" => {
            if let Some(path) = &call.path {
                run_read_file(path)
            } else {
                "Error: Missing 'path' argument for read_file".to_string()
            }
        }
        "write_file" => {
            if let (Some(path), Some(content)) = (&call.path, &call.content) {
                run_write_file(path, content)
            } else {
                "Error: Missing 'path' or 'content' argument for write_file".to_string()
            }
        }
        "search_web" => {
            if let Some(query) = &call.query {
                run_search_web(client, query).await
            } else {
                "Error: Missing 'query' argument for search_web".to_string()
            }
        }
        "fetch_url" => {
            if let Some(url) = &call.url {
                run_fetch_url(client, url).await
            } else {
                "Error: Missing 'url' argument for fetch_url".to_string()
            }
        }
        _ => format!("Error: Unknown tool '{}'", tool_name),
    }
}

pub async fn run_agent_loop(
    req: ChatRequest,
    client: &reqwest::Client,
    env_api_key: Option<String>,
) -> Result<ChatResponse> {
    let api_key = req.api_key.or(env_api_key);
    let model = req.model.unwrap_or_else(|| "gemini-1.5-flash".to_string());
    let temp = req.temperature.unwrap_or(0.7);

    // If no API key is provided, execute Demo Mode
    if api_key.is_none() {
        return Ok(run_demo_mode(&req.messages, client).await);
    }

    let key = api_key.unwrap();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, key
    );

    let system_instruction = "\
You are an advanced agentic coding and analysis assistant. You have access to the following tools:
- `calculator(expression)`: Solve mathematical equations. Returns the evaluated float or error. Example request: <tool_call>{\"tool\": \"calculator\", \"expression\": \"(12 + 8) * 5\"}</tool_call>
- `get_system_info()`: Retrieve OS, Architecture, Current Directory, and Uptime metrics. Example request: <tool_call>{\"tool\": \"get_system_info\"}</tool_call>
- `get_time_date()`: Get the current local date, time, and timezone. Example request: <tool_call>{\"tool\": \"get_time_date\"}</tool_call>
- `list_directory(path)`: List file names in a directory relative to workspace. Optional path argument. Example request: <tool_call>{\"tool\": \"list_directory\", \"path\": \"src\"}</tool_call>
- `read_file(path)`: Read the entire text content of a file relative to workspace. Example request: <tool_call>{\"tool\": \"read_file\", \"path\": \"src/main.rs\"}</tool_call>
- `write_file(path, content)`: Write or overwrite text content of a file relative to workspace. Example request: <tool_call>{\"tool\": \"write_file\", \"path\": \"src/temp.txt\", \"content\": \"Hello World!\"}</tool_call>
- `search_web(query)`: Search the web using DuckDuckGo search. Returns matching titles, URLs, and snippets. Example request: <tool_call>{\"tool\": \"search_web\", \"query\": \"Rust Axum routing tutorial\"}</tool_call>
- `fetch_url(url)`: Fetch and extract plain text content from a web page URL. Example request: <tool_call>{\"tool\": \"fetch_url\", \"url\": \"https://example.com\"}</tool_call>

To use a tool, you must respond with a JSON object enclosed in `<tool_call>` and `</tool_call>` tags.
Before making a tool call, or if you do not need a tool call, write out your detailed thinking process enclosed in `<thought>` and `</thought>` tags. Always output the tool_call tags on a separate line.

For example:
<thought>
I need to check the current directory contents to see the main source files. Let's run the list_directory tool.
</thought>
<tool_call>
{\"tool\": \"list_directory\"}
</tool_call>

Once you receive the tool result, analyze it, make further tool calls if needed, and when finished, output your final response outside of any `<thought>` or `<tool_call>` tags. Always explain the results clearly.";

    let mut conversation = req.messages.clone();
    let mut steps = Vec::new();
    let mut final_response = String::new();
    let mut loop_count = 0;
    const MAX_LOOPS: usize = 5;

    while loop_count < MAX_LOOPS {
        loop_count += 1;
        info!("Running agent loop iteration {}", loop_count);

        // Map conversation into Gemini API format
        let mut contents = Vec::new();
        for msg in &conversation {
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

        let response = client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Gemini API")?;

        let status = response.status();
        if !status.is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Gemini API returned status {}: {}", status, err_text));
        }

        let resp_json: serde_json::Value = response.json().await.context("Failed to parse Gemini API response")?;
        
        // Extract output text
        let output_text = resp_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Empty response from Gemini API: {:?}", resp_json))?
            .to_string();

        info!("Agent output: {}", output_text);

        // Parse thought if present
        if let Some(start_thought) = output_text.find("<thought>") {
            if let Some(end_thought) = output_text.find("</thought>") {
                let thought_content = output_text[start_thought + 9..end_thought].trim().to_string();
                steps.push(AgentStep {
                    title: "Thinking Process".to_string(),
                    log: thought_content,
                    status: "success".to_string(),
                });
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
                    let step_title = format!("Executing Tool: {}", tool_name);
                    let mut step_log = format!("Arguments: {}\n", tool_json_str.trim());
                    
                    steps.push(AgentStep {
                        title: step_title.clone(),
                        log: step_log.clone() + "Running...",
                        status: "pending".to_string(),
                    });
                    
                    let step_index = steps.len() - 1;
                    
                    // Execute tool
                    let tool_result = execute_tool(&tool_name, &tool_call, client).await;
                    
                    // Update step
                    step_log.push_str(&format!("\nResult:\n{}", tool_result));
                    steps[step_index].log = step_log;
                    steps[step_index].status = "success".to_string();

                    // Append model's thought & tool call to history
                    conversation.push(Message {
                        role: "model".to_string(),
                        content: output_text.clone(),
                    });

                    // Append tool result as user input to history
                    conversation.push(Message {
                        role: "user".to_string(),
                        content: format!("Tool result: {}", tool_result),
                    });
                }
            }
        }

        if !has_tool_call {
            // No tool call, means the agent finished and returned its final response.
            // Strip tags from final response to make it clean
            let clean_response = output_text
                .replace("<thought>", "")
                .replace("</thought>", "")
                .split("</thought>")
                .last()
                .unwrap_or(&output_text)
                .trim()
                .to_string();
            
            final_response = clean_response;
            break;
        }
    }

    if final_response.is_empty() {
        final_response = "Agent loop exceeded maximum turns without final response.".to_string();
    }

    Ok(ChatResponse {
        steps,
        response: final_response,
    })
}

// --- DEMO MODE MOCK ---
async fn run_demo_mode(messages: &[Message], client: &reqwest::Client) -> ChatResponse {
    let last_user_msg = messages
        .iter()
        .filter(|m| m.role == "user")
        .last()
        .map(|m| m.content.to_lowercase())
        .unwrap_or_default();

    let mut steps = Vec::new();
    let response: String;

    if last_user_msg.contains("sistem") || last_user_msg.contains("system") || last_user_msg.contains("info") {
        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: "User is asking for system information. I will call the `get_system_info()` tool to retrieve operating system metrics, working directory, and uptime.".to_string(),
            status: "success".to_string(),
        });
        
        let sys_info = run_system_info();
        steps.push(AgentStep {
            title: "Executing Tool: get_system_info".to_string(),
            log: format!("Arguments: {{}}\n\nResult:\n{}", sys_info),
            status: "success".to_string(),
        });

        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: "I have retrieved the system information. I will now present it clearly to the user, highlighting the operating system, current directory, and server uptime.".to_string(),
            status: "success".to_string(),
        });

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

        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: format!("User requested a math calculation. I will parse the expression `{}` and execute the `calculator` tool to solve it.", expr),
            status: "success".to_string(),
        });

        let calc_res = run_calculator(&expr);
        steps.push(AgentStep {
            title: "Executing Tool: calculator".to_string(),
            log: format!("Arguments: {{\n  \"expression\": \"{}\"\n}}\n\nResult:\n{}", expr, calc_res),
            status: "success".to_string(),
        });

        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: "Calculation completed successfully. I will explain the solution steps and provide the final result to the user.".to_string(),
            status: "success".to_string(),
        });

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
        
        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: format!("User requested to read the file `{}`. I will execute the `read_file` tool to retrieve its content.", path),
            status: "success".to_string(),
        });
        
        let content = run_read_file(&path);
        
        steps.push(AgentStep {
            title: "Executing Tool: read_file".to_string(),
            log: format!("Arguments: {{\n  \"path\": \"{}\"\n}}\n\nResult:\n(Successfully read file content)", path),
            status: "success".to_string(),
        });

        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: "Content of file has been read. I will now display it inside a formatted markdown code block for the user.".to_string(),
            status: "success".to_string(),
        });

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

        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: format!("User requested to write a file at `{}`. I will run `write_file` to save the content.", path),
            status: "success".to_string(),
        });

        let write_res = run_write_file(&path, &mock_content);

        steps.push(AgentStep {
            title: "Executing Tool: write_file".to_string(),
            log: format!("Arguments: {{\n  \"path\": \"{}\",\n  \"content\": \"{}\"\n}}\n\nResult:\n{}", path, mock_content, write_res),
            status: "success".to_string(),
        });

        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: "File has been written. I will report the success to the user.".to_string(),
            status: "success".to_string(),
        });

        response = format!(
            "Berhasil menulis file di `{}` (Mode Demo):\n\n\
            **Status**: `{}`\n\n\
            *Isi yang ditulis:*\n\
            ```text\n\
            {}\n\
            ```",
            path, write_res, mock_content
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
        
        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: format!("User wants to search the web for `{}`. I will call `search_web` to get matching web pages.", query),
            status: "success".to_string(),
        });

        let search_res = run_search_web(client, query).await;

        steps.push(AgentStep {
            title: "Executing Tool: search_web".to_string(),
            log: format!("Arguments: {{\n  \"query\": \"{}\"\n}}\n\nResult:\n(Fetched search results)", query),
            status: "success".to_string(),
        });

        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: "Web search results retrieved. I will format them nicely in markdown for presentation.".to_string(),
            status: "success".to_string(),
        });

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

        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: format!("User provided a URL: `{}`. I will execute the `fetch_url` tool to scrape and read the web page text.", url_found),
            status: "success".to_string(),
        });

        let fetch_res = run_fetch_url(client, &url_found).await;

        steps.push(AgentStep {
            title: "Executing Tool: fetch_url".to_string(),
            log: format!("Arguments: {{\n  \"url\": \"{}\"\n}}\n\nResult:\n(Successfully fetched page content)", url_found),
            status: "success".to_string(),
        });

        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: "Web page content has been fetched. I will now present a summary or the first few paragraphs to the user.".to_string(),
            status: "success".to_string(),
        });

        response = format!(
            "Berikut adalah konten teks dari URL `{}` (Mode Demo):\n\n\
            ```text\n\
            {}\n\
            ```",
            url_found, fetch_res
        );
    } else if last_user_msg.contains("file") || last_user_msg.contains("direktori") || last_user_msg.contains("ls") || last_user_msg.contains("list") {
        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: "User wants to see the files inside the directory. I will call `list_directory()` for the current directory root.".to_string(),
            status: "success".to_string(),
        });

        let dir_res = run_list_directory(None);
        steps.push(AgentStep {
            title: "Executing Tool: list_directory".to_string(),
            log: format!("Arguments: {{}}\n\nResult:\n{}", dir_res),
            status: "success".to_string(),
        });

        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: "Directory contents retrieved. I will format the directory contents list neatly in markdown code block for readability.".to_string(),
            status: "success".to_string(),
        });

        response = format!(
            "Berikut adalah daftar file di direktori kerja server agen (Mode Demo):\n\n\
            ```text\n\
            {}\n\
            ```\n\n\
            *Masukkan Gemini API Key untuk menguji agen otonom sesungguhnya yang dapat menelusuri file dan memecahkan kode Anda secara dinamis!*",
            dir_res
        );
    } else {
        steps.push(AgentStep {
            title: "Thinking Process".to_string(),
            log: "User greeted me or asked a general question. Since I am in Demo Mode, I will introduce my capabilities and explain how the user can activate the full Agent features with their Gemini API key.".to_string(),
            status: "success".to_string(),
        });

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
               * 🔗 **Scrape URL**: Mengambil data teks dari halaman web.\n\n\
            **Cara Mengaktifkan Fitur Penuh:**\n\
            * Klik ikon **Pengaturan (Gir)** ⚙️ di pojok kiri bawah UI.\n\
            * Masukkan **Gemini API Key** Anda.\n\
            * Pilih model (seperti `gemini-2.0-flash` atau `gemini-1.5-pro`).\n\
            * Mulai ajukan pertanyaan kompleks dan perhatikan saya bekerja otonom menggunakan perkakas saya!"
        );
    }

    ChatResponse { steps, response }
}
