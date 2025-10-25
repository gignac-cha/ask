# Automation Loop - Code Reference

Quick reference for key functions and patterns in the autonomous browser automation implementation.

## Table of Contents
1. [Core Loop Function](#core-loop-function)
2. [Playwright Integration](#playwright-integration)
3. [Gemini API Integration](#gemini-api-integration)
4. [Prompt Engineering](#prompt-engineering)
5. [Error Recovery](#error-recovery)
6. [Background Task Spawning](#background-task-spawning)

---

## Core Loop Function

### Main Automation Loop
**Location**: `/home/user/ask/src-tauri/src/lib.rs:411-554`

```rust
async fn run_automation_loop(
    app: tauri::AppHandle,
    query: String,
    api_key: String,
    model_name: String,
) {
    const MAX_ITERATIONS: usize = 10;
    let mut history: Vec<ActionHistory> = Vec::new();

    // Get initial action from Gemini
    let initial_prompt = format!(
        r#"You are a browser automation assistant. The user wants: '{}'

        Determine the FIRST action to take to achieve this goal.
        ..."#,
        query
    );

    let initial_response = call_gemini_api(initial_prompt, api_key.clone(), model_name.clone()).await?;
    let mut current_action = extract_function_call(&initial_response)?;

    // Main loop
    for iteration in 0..MAX_ITERATIONS {
        // Check for completion
        if current_action.name == "finish" {
            let final_answer = current_action.args["answer"].as_str().unwrap_or("Done");
            println!("Task completed: {}", final_answer);
            break;
        }

        // Execute action
        let action_json = serde_json::json!({
            "function": current_action.name,
            "args": current_action.args
        }).to_string();

        let step_result = execute_playwright_step(action_json).await?;

        // Update history
        history.push(ActionHistory {
            action: current_action.name.clone(),
            args: current_action.args.clone(),
            result: step_result["message"].as_str().unwrap_or("Success").to_string(),
        });

        // Get next action
        let next_prompt = create_next_step_prompt(&query, &history, &step_result["dom"]);
        let next_response = call_gemini_api(next_prompt, api_key.clone(), model_name.clone()).await?;
        current_action = extract_function_call(&next_response)?;
    }

    // Cleanup
    cleanup_playwright().await?;
}
```

---

## Playwright Integration

### Execute Single Step
**Location**: `/home/user/ask/src-tauri/src/lib.rs:246-287`

```rust
async fn execute_playwright_step(action_json: String) -> Result<serde_json::Value, String> {
    // Find script path
    let script_path = env::current_dir()?
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("scripts").join("playwright-step-executor.mjs"))
        .ok_or("Could not find scripts directory")?;

    // Execute Node.js script
    let output = Command::new("node")
        .arg(script_path)
        .arg(&action_json)
        .output()
        .map_err(|e| format!("Failed to execute playwright script: {}", e))?;

    // Parse result
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse Playwright output: {}", e))?;

    Ok(result)
}
```

### Cleanup Browser
**Location**: `/home/user/ask/src-tauri/src/lib.rs:289-309`

```rust
async fn cleanup_playwright() -> Result<(), String> {
    let script_path = env::current_dir()?
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("scripts").join("playwright-step-executor.mjs"))
        .ok_or("Could not find scripts directory")?;

    if script_path.exists() {
        Command::new("node")
            .arg(script_path)
            .arg("{}")
            .arg("cleanup")
            .output()
            .ok();
    }

    Ok(())
}
```

### Expected Playwright Response
```json
{
  "success": true,
  "action": "goToURL",
  "timestamp": "2025-10-25T12:00:00.000Z",
  "message": "Navigated to https://example.com",
  "dom": {
    "url": "https://example.com",
    "title": "Example Domain",
    "mainText": "This domain is for use in illustrative examples...",
    "headings": ["Example Domain"],
    "links": [
      {"text": "More information...", "href": "https://www.iana.org/domains/example"}
    ],
    "inputs": []
  },
  "screenshot": "iVBORw0KGgoAAAANSUhEUgA...",
  "currentUrl": "https://example.com"
}
```

---

## Gemini API Integration

### Call Gemini API
**Location**: `/home/user/ask/src-tauri/src/lib.rs:163-215`

```rust
async fn call_gemini_api(
    prompt: String,
    api_key: String,
    model: String,
) -> Result<GeminiResponse, String> {
    let tools = create_browser_tools();
    let model_id = model.strip_prefix("models/").unwrap_or(&model);

    let request_body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part { text: prompt }],
        }],
        tools,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model_id, api_key
        ))
        .json(&request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    let gemini_response: GeminiResponse = serde_json::from_str(&response_text)?;

    Ok(gemini_response)
}
```

### Extract Function Call
**Location**: `/home/user/ask/src-tauri/src/lib.rs:217-244`

```rust
fn extract_function_call(response: &GeminiResponse) -> Result<FunctionCall, String> {
    if let Some(candidate) = response.candidates.first() {
        // Look for function call in response parts
        for part in &candidate.content.parts {
            if let ResponsePart::FunctionCall { function_call } = part {
                return Ok(function_call.clone());
            }
        }

        // Fallback: check for text indicating completion
        for part in &candidate.content.parts {
            if let ResponsePart::Text { text } = part {
                if text.to_lowercase().contains("task complete") {
                    return Ok(FunctionCall {
                        name: "finish".to_string(),
                        args: serde_json::json!({"answer": text}),
                    });
                }
            }
        }

        Err("No function call found in response".to_string())
    } else {
        Err("No candidates in Gemini response".to_string())
    }
}
```

### Browser Tools Definition
**Location**: `/home/user/ask/src-tauri/src/lib.rs:85-166`

```rust
fn create_browser_tools() -> Vec<Tool> {
    vec![Tool {
        function_declarations: vec![
            FunctionDeclaration {
                name: "goToURL".to_string(),
                description: "Navigate to a specific URL in the browser".to_string(),
                parameters: Parameters {
                    r#type: "object".to_string(),
                    properties: serde_json::json!({
                        "url": {"type": "string", "description": "The URL to navigate to"}
                    }),
                    required: vec!["url".to_string()],
                },
            },
            FunctionDeclaration {
                name: "click".to_string(),
                description: "Click on an element matching the given CSS selector".to_string(),
                parameters: Parameters {
                    r#type: "object".to_string(),
                    properties: serde_json::json!({
                        "selector": {"type": "string", "description": "CSS selector"}
                    }),
                    required: vec!["selector".to_string()],
                },
            },
            FunctionDeclaration {
                name: "typeText".to_string(),
                description: "Type text into an input element".to_string(),
                parameters: Parameters {
                    r#type: "object".to_string(),
                    properties: serde_json::json!({
                        "selector": {"type": "string"},
                        "text": {"type": "string"}
                    }),
                    required: vec!["selector".to_string(), "text".to_string()],
                },
            },
            FunctionDeclaration {
                name: "getText".to_string(),
                description: "Extract text content from an element".to_string(),
                parameters: Parameters {
                    r#type: "object".to_string(),
                    properties: serde_json::json!({
                        "selector": {"type": "string"}
                    }),
                    required: vec!["selector".to_string()],
                },
            },
            FunctionDeclaration {
                name: "finish".to_string(),
                description: "Complete the task and return the final answer".to_string(),
                parameters: Parameters {
                    r#type: "object".to_string(),
                    properties: serde_json::json!({
                        "answer": {"type": "string", "description": "Final answer to return"}
                    }),
                    required: vec!["answer".to_string()],
                },
            },
        ],
    }]
}
```

---

## Prompt Engineering

### Create Next Step Prompt
**Location**: `/home/user/ask/src-tauri/src/lib.rs:311-422`

```rust
fn create_next_step_prompt(
    original_goal: &str,
    history: &[ActionHistory],
    dom_info: &serde_json::Value,
) -> String {
    // Extract page state
    let current_url = dom_info["url"].as_str().unwrap_or("unknown");
    let page_title = dom_info["title"].as_str().unwrap_or("unknown");
    let main_text = dom_info["mainText"]
        .as_str()
        .unwrap_or("")
        .chars()
        .take(1000)  // Limit to prevent token overflow
        .collect::<String>();

    // Format links
    let links_info = if let Some(links) = dom_info["links"].as_array() {
        links.iter()
            .take(10)
            .filter_map(|link| {
                Some(format!("- {} ({})",
                    link["text"].as_str()?,
                    link["href"].as_str()?
                ))
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        "No links found".to_string()
    };

    // Format inputs
    let inputs_info = if let Some(inputs) = dom_info["inputs"].as_array() {
        inputs.iter()
            .filter_map(|input| {
                Some(format!("- Type: {}, Name: {}, ID: {}",
                    input["type"].as_str()?,
                    input["name"].as_str().unwrap_or(""),
                    input["id"].as_str().unwrap_or("")
                ))
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        "No inputs found".to_string()
    };

    // Format history
    let history_text = if history.is_empty() {
        "None yet - this is the first action.".to_string()
    } else {
        history.iter()
            .enumerate()
            .map(|(i, h)| format!("{}. Action: {} - Result: {}", i + 1, h.action, h.result))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(r#"You are a browser automation assistant.

ORIGINAL GOAL: {}

CURRENT PAGE STATE:
- URL: {}
- Page Title: {}
- Main Content Preview:
{}

- Available Links:
{}

- Form Inputs:
{}

ACTIONS COMPLETED SO FAR:
{}

INSTRUCTIONS:
Based on the current page state and the original goal, determine the NEXT SINGLE action.

If goal is achieved, return: {{"action": "finish", "args": {{"answer": "..."}}}}

Otherwise, choose ONE: goToURL, click, typeText, getText

Think step-by-step about what action will best progress toward the goal."#,
        original_goal, current_url, page_title, main_text,
        links_info, inputs_info, history_text
    )
}
```

---

## Error Recovery

### Playwright Step with Recovery
**Location**: `/home/user/ask/src-tauri/src/lib.rs:481-503`

```rust
// In run_automation_loop
let step_result = match execute_playwright_step(action_json).await {
    Ok(result) => result,
    Err(e) => {
        eprintln!("Playwright step failed: {}", e);

        // Try to recover by asking Gemini for alternative
        let recovery_prompt = format!(
            "The previous action failed with error: {}. \
             Original goal: {}. \
             Suggest an alternative approach or finish if the goal cannot be achieved.",
            e, query
        );

        if let Ok(recovery_response) = call_gemini_api(
            recovery_prompt,
            api_key.clone(),
            model_name.clone()
        ).await {
            if let Ok(alt_action) = extract_function_call(&recovery_response) {
                current_action = alt_action;
                continue;  // Try alternative action
            }
        }

        eprintln!("Could not recover from error, aborting automation");
        break;  // Give up
    }
};
```

---

## Background Task Spawning

### Refactored handle_user_query
**Location**: `/home/user/ask/src-tauri/src/lib.rs:556-614`

```rust
#[tauri::command]
async fn handle_user_query(app: tauri::AppHandle, query: String) -> Result<String, String> {
    use tauri_plugin_store::StoreExt;

    // Load and validate settings
    let store = app.store("settings.json")?;
    let settings_json = store
        .get("app-settings")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "{}".to_string());

    let settings: serde_json::Value = serde_json::from_str(&settings_json)?;
    let api_key = settings["apiKey"].as_str().unwrap_or("").to_string();
    let model_name = settings["modelName"].as_str().unwrap_or("").to_string();

    if api_key.is_empty() {
        return Err("API key not configured.".to_string());
    }
    if model_name.is_empty() {
        return Err("Model not selected.".to_string());
    }

    // Spawn background task - DOES NOT BLOCK
    let app_handle = app.clone();
    let query_clone = query.clone();
    tokio::spawn(async move {
        run_automation_loop(app_handle, query_clone, api_key, model_name).await;
    });

    // Return immediately
    Ok(format!(
        "Task started: I'm working on '{}'. \
         The automation will run in the background and complete autonomously.",
        query
    ))
}
```

---

## Data Structures

### ActionHistory
```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ActionHistory {
    action: String,              // "goToURL", "click", etc.
    args: serde_json::Value,     // { "url": "..." } or { "selector": "..." }
    result: String,              // "Navigated to ..." or "Clicked ..."
}
```

### FunctionCall
```rust
#[derive(Serialize, Deserialize, Debug)]
struct FunctionCall {
    name: String,               // Action name
    args: serde_json::Value,    // Action arguments
}
```

---

## Usage Example

### From Frontend
```typescript
// In ChatPage.tsx
const response = await invoke<string>("handle_user_query", {
  query: "What's on Hacker News front page?"
});

console.log(response);
// Output: "Task started: I'm working on 'What's on Hacker News front page?'. ..."
```

### Expected Console Output (Backend)
```
Received query: What's on Hacker News front page?
Starting automation loop for query: What's on Hacker News front page?

=== Iteration 1 ===
Action: goToURL with args: {"url":"https://news.ycombinator.com"}
Executing Playwright action: {"function":"goToURL","args":{"url":"https://news.ycombinator.com"}}
[Executor] Executing: goToURL {"url":"https://news.ycombinator.com"}

=== Iteration 2 ===
Action: finish with args: {"answer":"Top stories on Hacker News:\n1. Show HN: ..."}
Task completed: Top stories on Hacker News:
1. Show HN: I built a...
2. Ask HN: Best practices...
...

Cleaning up browser...
Automation loop completed
```

---

## Testing Commands

### Build
```bash
cd /home/user/ask/src-tauri
cargo build
```

### Run Development Server
```bash
cd /home/user/ask
npm run tauri dev
```

### Check for Errors
```bash
cd /home/user/ask/src-tauri
cargo check
```

---

## Common Patterns

### Pattern 1: Simple Navigation
```rust
// Action: goToURL("https://example.com")
// Result: Browser navigates, returns DOM
// Next: Analyze DOM and decide next action
```

### Pattern 2: Search Flow
```rust
// 1. goToURL("https://google.com")
// 2. typeText("input[name='q']", "search query")
// 3. click("button[type='submit']")
// 4. getText(".search-result")
// 5. finish("Here are the results...")
```

### Pattern 3: Information Extraction
```rust
// 1. goToURL("https://news.site.com")
// 2. getText("article h2") or analyze DOM
// 3. finish("Here's the summary...")
```

---

## Debugging Tips

1. **Check logs**: All actions are logged with `println!`
2. **Inspect Gemini prompts**: Logged before each API call
3. **Verify Playwright output**: Check stdout/stderr
4. **Monitor history**: Each iteration logs action history
5. **Watch for cleanup**: Ensures browser closes properly

---

## Key Files

- **Main implementation**: `/home/user/ask/src-tauri/src/lib.rs`
- **Playwright executor**: `/home/user/ask/scripts/playwright-step-executor.mjs`
- **Frontend integration**: `/home/user/ask/src/pages/ChatPage.tsx`
- **Documentation**:
  - `/home/user/ask/AUTOMATION_LOOP_IMPLEMENTATION.md`
  - `/home/user/ask/AUTOMATION_ARCHITECTURE.md`
  - `/home/user/ask/CODE_REFERENCE.md` (this file)
