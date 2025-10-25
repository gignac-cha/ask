use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;
use tauri::Emitter;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Gemini API structures
#[derive(Serialize, Deserialize, Debug)]
struct GeminiRequest {
    contents: Vec<Content>,
    tools: Vec<Tool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Part {
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Tool {
    function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FunctionDeclaration {
    name: String,
    description: String,
    parameters: Parameters,
}

#[derive(Serialize, Deserialize, Debug)]
struct Parameters {
    r#type: String,
    properties: serde_json::Value,
    required: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Candidate {
    content: ResponseContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum ResponsePart {
    Text { text: String },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: FunctionCall
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct FunctionCall {
    name: String,
    args: serde_json::Value,
}

fn create_browser_tools() -> Vec<Tool> {
    vec![Tool {
        function_declarations: vec![
            FunctionDeclaration {
                name: "goToURL".to_string(),
                description: "Navigate to a specific URL in the browser".to_string(),
                parameters: Parameters {
                    r#type: "object".to_string(),
                    properties: serde_json::json!({
                        "url": {
                            "type": "string",
                            "description": "The URL to navigate to"
                        }
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
                        "selector": {
                            "type": "string",
                            "description": "CSS selector of the element to click"
                        }
                    }),
                    required: vec!["selector".to_string()],
                },
            },
            FunctionDeclaration {
                name: "getText".to_string(),
                description: "Extract text content from an element matching the given CSS selector"
                    .to_string(),
                parameters: Parameters {
                    r#type: "object".to_string(),
                    properties: serde_json::json!({
                        "selector": {
                            "type": "string",
                            "description": "CSS selector of the element to extract text from"
                        }
                    }),
                    required: vec!["selector".to_string()],
                },
            },
            FunctionDeclaration {
                name: "typeText".to_string(),
                description: "Type text into an input element matching the given CSS selector"
                    .to_string(),
                parameters: Parameters {
                    r#type: "object".to_string(),
                    properties: serde_json::json!({
                        "selector": {
                            "type": "string",
                            "description": "CSS selector of the input element"
                        },
                        "text": {
                            "type": "string",
                            "description": "Text to type into the input"
                        }
                    }),
                    required: vec!["selector".to_string(), "text".to_string()],
                },
            },
            FunctionDeclaration {
                name: "finish".to_string(),
                description: "Complete the task and return the final answer to the user".to_string(),
                parameters: Parameters {
                    r#type: "object".to_string(),
                    properties: serde_json::json!({
                        "answer": {
                            "type": "string",
                            "description": "The final answer or summary to return to the user"
                        }
                    }),
                    required: vec!["answer".to_string()],
                },
            },
        ],
    }]
}

// Helper struct to track action history
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ActionHistory {
    action: String,
    args: serde_json::Value,
    result: String,
}

// Event payload structs for real-time UI feedback
#[derive(Clone, Serialize)]
struct AutomationStatus {
    message: String,
    step: usize,
    total_steps: usize,
    current_url: Option<String>,
}

#[derive(Clone, Serialize)]
struct AutomationComplete {
    success: bool,
    answer: Option<String>,
    error: Option<String>,
}

#[derive(Clone, Serialize)]
struct BrowserViewUpdate {
    screenshot: String, // base64
    url: String,
}

// Call Gemini API with a prompt and return the parsed response
async fn call_gemini_api(
    prompt: String,
    api_key: String,
    model: String,
) -> Result<GeminiResponse, String> {
    let tools = create_browser_tools();

    // Remove "models/" prefix if present
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
        .await
        .map_err(|e| format!("Failed to call Gemini API: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Gemini API error ({}): {}", status, error_text));
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    println!("Gemini API Response: {}", response_text);

    let gemini_response: GeminiResponse = serde_json::from_str(&response_text).map_err(|e| {
        format!(
            "Failed to parse Gemini response: {}. Response was: {}",
            e, response_text
        )
    })?;

    Ok(gemini_response)
}

// Extract function call from Gemini response
fn extract_function_call(response: &GeminiResponse) -> Result<FunctionCall, String> {
    if let Some(candidate) = response.candidates.first() {
        for part in &candidate.content.parts {
            if let ResponsePart::FunctionCall { function_call } = part {
                return Ok(function_call.clone());
            }
        }
        // If no function call, check for text that might indicate completion
        for part in &candidate.content.parts {
            if let ResponsePart::Text { text } = part {
                if text.to_lowercase().contains("task complete")
                    || text.to_lowercase().contains("finished")
                {
                    return Ok(FunctionCall {
                        name: "finish".to_string(),
                        args: serde_json::json!({
                            "answer": text
                        }),
                    });
                }
            }
        }
        Err("No function call found in response".to_string())
    } else {
        Err("No candidates in Gemini response".to_string())
    }
}

// Helper function to find project root directory
fn find_project_root() -> Result<std::path::PathBuf, String> {
    let mut current = env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

    // Traverse up until we find a directory containing "scripts" and "src-tauri"
    loop {
        let scripts_dir = current.join("scripts");
        let src_tauri_dir = current.join("src-tauri");

        if scripts_dir.exists() && src_tauri_dir.exists() {
            return Ok(current);
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return Err("Could not find project root directory".to_string()),
        }
    }
}

// Execute a single Playwright step
async fn execute_playwright_step(action_json: String) -> Result<serde_json::Value, String> {
    let project_root = find_project_root()?;
    let script_path = project_root.join("scripts").join("playwright-step-executor.mjs");

    if !script_path.exists() {
        return Err(format!(
            "Playwright executor script not found at: {}",
            script_path.display()
        ));
    }

    println!("Executing Playwright action: {}", action_json);

    let output = Command::new("node")
        .arg(script_path)
        .arg(&action_json)
        .output()
        .map_err(|e| format!("Failed to execute playwright script: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stderr.is_empty() {
        println!("Playwright stderr: {}", stderr);
    }

    if !output.status.success() {
        return Err(format!("Playwright execution failed: {}", stderr));
    }

    let result: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse Playwright output: {}. Output was: {}", e, stdout))?;

    Ok(result)
}

// Cleanup Playwright browser
async fn cleanup_playwright() -> Result<(), String> {
    let project_root = find_project_root()?;
    let script_path = project_root.join("scripts").join("playwright-step-executor.mjs");

    if script_path.exists() {
        let _ = Command::new("node")
            .arg(script_path)
            .arg("{}")
            .arg("cleanup")
            .output();
    }

    Ok(())
}

// Check if the page contains CAPTCHA or bot detection
fn is_captcha_page(page_title: &str) -> bool {
    let title_lower = page_title.to_lowercase();

    // Check for common CAPTCHA and bot detection phrases
    let captcha_keywords = [
        "i'm not a robot",
        "i'm not a robot",  // Alternative apostrophe
        "before you continue",
        "captcha",
        "human verification",
        "verify you are human",
        "confirm you're not a robot",
        "security check",
        "unusual traffic",
        "automated requests",
        "bot detection",
        "suspicious activity",
    ];

    captcha_keywords.iter().any(|&keyword| title_lower.contains(keyword))
}

// Generate prompt for next step based on current state
fn create_next_step_prompt(
    original_goal: &str,
    history: &[ActionHistory],
    dom_info: &serde_json::Value,
) -> String {
    let current_url = dom_info["url"].as_str().unwrap_or("unknown");
    let page_title = dom_info["title"].as_str().unwrap_or("unknown");
    let main_text = dom_info["mainText"]
        .as_str()
        .unwrap_or("")
        .chars()
        .take(1000)
        .collect::<String>();

    let links_info = if let Some(links) = dom_info["links"].as_array() {
        links
            .iter()
            .take(10)
            .filter_map(|link| {
                let text = link["text"].as_str()?;
                let href = link["href"].as_str()?;
                Some(format!("- {} ({})", text, href))
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        "No links found".to_string()
    };

    let inputs_info = if let Some(inputs) = dom_info["inputs"].as_array() {
        inputs
            .iter()
            .filter_map(|input| {
                let input_type = input["type"].as_str()?;
                let name = input["name"].as_str().unwrap_or("");
                let id = input["id"].as_str().unwrap_or("");
                Some(format!(
                    "- Type: {}, Name: {}, ID: {}",
                    input_type, name, id
                ))
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        "No inputs found".to_string()
    };

    let history_text = if history.is_empty() {
        "None yet - this is the first action.".to_string()
    } else {
        history
            .iter()
            .enumerate()
            .map(|(i, h)| format!("{}. Action: {} - Result: {}", i + 1, h.action, h.result))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"You are a browser automation assistant helping to achieve a specific goal.

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
Based on the current page state and the original goal, determine the NEXT SINGLE action to take.

If you believe the goal has been achieved and you have gathered the necessary information, return a "finish" action with the final answer.

Example finish action:
{{"action": "finish", "args": {{"answer": "Here is the information you requested: ..."}}}}

Otherwise, choose ONE of these actions:
- goToURL: Navigate to a URL
- click: Click an element (use CSS selector)
- typeText: Type into an input field (use CSS selector and text)
- getText: Extract text from an element (use CSS selector)

Think step-by-step about what action will best progress toward the goal.
Choose wisely and act decisively."#,
        original_goal, current_url, page_title, main_text, links_info, inputs_info, history_text
    )
}

// Main automation loop that runs in background
async fn run_automation_loop(
    app: tauri::AppHandle,
    query: String,
    api_key: String,
    model_name: String,
) {
    const MAX_ITERATIONS: usize = 10;
    let mut history: Vec<ActionHistory> = Vec::new();

    println!("Starting automation loop for query: {}", query);

    // Emit initial status
    let _ = app.emit("automation-status", AutomationStatus {
        message: "Starting automation...".to_string(),
        step: 0,
        total_steps: MAX_ITERATIONS,
        current_url: None,
    });

    // Initial call to Gemini
    let initial_prompt = format!(
        r#"You are a browser automation assistant. The user wants: '{}'

Determine the FIRST action to take to achieve this goal.

You have access to these browser actions:
- goToURL: Navigate to a URL
- click: Click an element by CSS selector
- typeText: Type text into an input by CSS selector
- getText: Extract text from an element by CSS selector

Respond with the first action to take. For example, if the user wants information from a website, start by navigating to that website with goToURL.

Think about what website would be most helpful for this query and navigate there first."#,
        query
    );

    // Emit status: Planning first action
    let _ = app.emit("automation-status", AutomationStatus {
        message: "Planning first action...".to_string(),
        step: 1,
        total_steps: MAX_ITERATIONS,
        current_url: None,
    });

    let initial_response = match call_gemini_api(initial_prompt, api_key.clone(), model_name.clone()).await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Failed to get initial action from Gemini: {}", e);
            let _ = app.emit("automation-complete", AutomationComplete {
                success: false,
                answer: None,
                error: Some(format!("Failed to get initial action: {}", e)),
            });
            return;
        }
    };

    let mut current_action = match extract_function_call(&initial_response) {
        Ok(action) => action,
        Err(e) => {
            eprintln!("Failed to extract initial action: {}", e);
            let _ = app.emit("automation-complete", AutomationComplete {
                success: false,
                answer: None,
                error: Some(format!("Failed to extract initial action: {}", e)),
            });
            return;
        }
    };

    // Main automation loop
    for iteration in 0..MAX_ITERATIONS {
        println!("\n=== Iteration {} ===", iteration + 1);
        println!("Action: {} with args: {:?}", current_action.name, current_action.args);

        // Check if we're done
        if current_action.name == "finish" {
            let final_answer = current_action.args["answer"]
                .as_str()
                .unwrap_or("Task completed successfully")
                .to_string();
            println!("Task completed: {}", final_answer);

            // Emit completion event
            let _ = app.emit("automation-complete", AutomationComplete {
                success: true,
                answer: Some(final_answer),
                error: None,
            });
            break;
        }

        // Emit status: Executing action
        let action_message = format!(
            "Executing: {} {}",
            current_action.name,
            serde_json::to_string(&current_action.args).unwrap_or_default()
        );
        let _ = app.emit("automation-status", AutomationStatus {
            message: action_message,
            step: iteration + 1,
            total_steps: MAX_ITERATIONS,
            current_url: None,
        });

        // Execute the action via Playwright
        let action_json = serde_json::json!({
            "function": current_action.name,
            "args": current_action.args
        })
        .to_string();

        let step_result = match execute_playwright_step(action_json).await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Playwright step failed: {}", e);

                // Try to recover by asking Gemini for alternative approach
                let recovery_prompt = format!(
                    "The previous action failed with error: {}. \
                     Original goal: {}. \
                     Suggest an alternative approach or finish if the goal cannot be achieved.",
                    e, query
                );

                if let Ok(recovery_response) = call_gemini_api(recovery_prompt, api_key.clone(), model_name.clone()).await {
                    if let Ok(alt_action) = extract_function_call(&recovery_response) {
                        current_action = alt_action;
                        continue;
                    }
                }

                eprintln!("Could not recover from error, aborting automation");
                let _ = app.emit("automation-complete", AutomationComplete {
                    success: false,
                    answer: None,
                    error: Some(e),
                });
                break;
            }
        };

        // Record this action in history
        let result_summary = if step_result["success"].as_bool().unwrap_or(false) {
            step_result["message"].as_str().unwrap_or("Success").to_string()
        } else {
            format!(
                "Failed: {}",
                step_result["error"].as_str().unwrap_or("Unknown error")
            )
        };

        history.push(ActionHistory {
            action: current_action.name.clone(),
            args: current_action.args.clone(),
            result: result_summary.clone(),
        });

        // Get DOM info for next decision
        let dom_info = step_result["dom"].clone();

        // Extract current URL from DOM info
        let current_url = dom_info["url"].as_str().map(|s| s.to_string());

        // Check for CAPTCHA/bot detection BEFORE sending to Gemini
        let page_title = dom_info["title"].as_str().unwrap_or("");
        if is_captcha_page(page_title) {
            eprintln!("CAPTCHA/bot detection page detected: {}", page_title);
            let _ = app.emit("automation-complete", AutomationComplete {
                success: false,
                answer: None,
                error: Some(
                    "Error: Google 봇 감지(CAPTCHA) 페이지에 막혔습니다. 이 작업은 현재 수행할 수 없습니다."
                        .to_string(),
                ),
            });
            // Cleanup browser before exiting
            let _ = cleanup_playwright().await;
            return;
        }

        // Emit status: Action completed
        let completion_message = format!(
            "Completed: {}, Result: {}",
            current_action.name,
            result_summary
        );
        let _ = app.emit("automation-status", AutomationStatus {
            message: completion_message,
            step: iteration + 1,
            total_steps: MAX_ITERATIONS,
            current_url: current_url.clone(),
        });

        // Emit screenshot update if available
        if let Some(screenshot) = step_result["screenshot"].as_str() {
            if let Some(url) = current_url.as_ref() {
                let _ = app.emit("browser-view-update", BrowserViewUpdate {
                    screenshot: screenshot.to_string(),
                    url: url.clone(),
                });
            }
        }

        if dom_info.is_null() {
            eprintln!("No DOM info available, cannot continue");
            let _ = app.emit("automation-complete", AutomationComplete {
                success: false,
                answer: None,
                error: Some("No DOM info available".to_string()),
            });
            break;
        }

        // Emit status: Planning next action
        let _ = app.emit("automation-status", AutomationStatus {
            message: "Planning next action...".to_string(),
            step: iteration + 2,
            total_steps: MAX_ITERATIONS,
            current_url: current_url.clone(),
        });

        // Ask Gemini for next step
        let next_prompt = create_next_step_prompt(&query, &history, &dom_info);

        let next_response = match call_gemini_api(next_prompt, api_key.clone(), model_name.clone()).await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Failed to get next action from Gemini: {}", e);
                let _ = app.emit("automation-complete", AutomationComplete {
                    success: false,
                    answer: None,
                    error: Some(format!("Failed to get next action: {}", e)),
                });
                break;
            }
        };

        current_action = match extract_function_call(&next_response) {
            Ok(action) => action,
            Err(e) => {
                eprintln!("Failed to extract next action: {}", e);
                let _ = app.emit("automation-complete", AutomationComplete {
                    success: false,
                    answer: None,
                    error: Some(format!("Failed to extract next action: {}", e)),
                });
                break;
            }
        };
    }

    // Cleanup
    println!("Cleaning up browser...");
    let _ = cleanup_playwright().await;
    println!("Automation loop completed");
}

#[tauri::command]
async fn handle_user_query(app: tauri::AppHandle, query: String) -> Result<String, String> {
    use tauri_plugin_store::StoreExt;

    println!("Received query: {}", query);

    // Load settings from store
    let store = app
        .store("settings.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;

    let settings_json = store
        .get("app-settings")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "{}".to_string());

    // Parse settings
    let settings: serde_json::Value = serde_json::from_str(&settings_json)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;

    let api_key = settings["apiKey"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let model_name = settings["modelName"]
        .as_str()
        .unwrap_or("")
        .to_string();

    // Check if API key is set
    if api_key.is_empty() {
        return Err(
            "API key not configured. Please go to Settings and enter your Gemini API key."
                .to_string(),
        );
    }

    // Check if model is selected
    if model_name.is_empty() {
        return Err(
            "Model not selected. Please go to Settings, fetch available models, and select one."
                .to_string(),
        );
    }

    // Spawn background task for automation loop
    let app_handle = app.clone();
    let query_clone = query.clone();
    tokio::spawn(async move {
        run_automation_loop(app_handle, query_clone, api_key, model_name).await;
    });

    // Return immediately with confirmation message
    Ok(format!(
        "Task started: I'm working on '{}'. The automation will run in the background and complete autonomously.",
        query
    ))
}

#[tauri::command]
async fn execute_browser_actions(actions: String) -> Result<String, String> {
    println!("Executing browser actions: {}", actions);

    // Get the project root directory
    let project_root = find_project_root()?;
    let script_path = project_root.join("scripts").join("playwright-executor.mjs");

    if !script_path.exists() {
        return Err(format!(
            "Playwright executor script not found at: {}",
            script_path.display()
        ));
    }

    // Execute the Node.js Playwright script
    let output = Command::new("node")
        .arg(script_path)
        .arg(&actions)
        .output()
        .map_err(|e| format!("Failed to execute playwright script: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Playwright execution failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

#[tauri::command]
async fn fetch_gemini_models(api_key: String) -> Result<String, String> {
    println!("Fetching Gemini models with API key");

    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models?key={}",
        api_key
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch models: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API error ({}): {}", status, error_text));
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    Ok(response_text)
}

#[tauri::command]
async fn save_settings(app: tauri::AppHandle, settings: String) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;

    let store = app
        .store("settings.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;

    store.set("app-settings", serde_json::json!(settings));

    store
        .save()
        .map_err(|e| format!("Failed to save store: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn load_settings(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_store::StoreExt;

    let store = app
        .store("settings.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;

    let settings = store
        .get("app-settings")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "{}".to_string());

    Ok(settings)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            handle_user_query,
            execute_browser_actions,
            fetch_gemini_models,
            save_settings,
            load_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/*
 * ============================================================================
 * AUTONOMOUS BROWSER AUTOMATION LOOP - TESTING GUIDE
 * ============================================================================
 *
 * OVERVIEW:
 * The automation loop runs autonomously in the background, executing browser
 * actions based on AI decisions from Gemini API.
 *
 * EXPECTED FLOW FOR: "What's on Hacker News front page?"
 * ============================================================================
 *
 * 1. USER REQUEST:
 *    User types: "What's on Hacker News front page?"
 *    Frontend calls: invoke("handle_user_query", { query: "..." })
 *
 * 2. IMMEDIATE RESPONSE:
 *    handle_user_query returns immediately:
 *    "Task started: I'm working on 'What's on Hacker News front page?'.
 *     The automation will run in the background and complete autonomously."
 *
 * 3. BACKGROUND LOOP STARTS (via tokio::spawn):
 *
 *    Iteration 1:
 *    -----------
 *    - Gemini call: "Determine the FIRST action to achieve: What's on Hacker News front page?"
 *    - Gemini returns: { "function": "goToURL", "args": { "url": "https://news.ycombinator.com" } }
 *    - Execute via Playwright: Navigate to https://news.ycombinator.com
 *    - Playwright returns: {
 *        "success": true,
 *        "dom": {
 *          "url": "https://news.ycombinator.com",
 *          "title": "Hacker News",
 *          "mainText": "...",
 *          "links": [
 *            { "text": "Show HN: I built...", "href": "..." },
 *            { "text": "Ask HN: Best practices...", "href": "..." },
 *            ...
 *          ]
 *        },
 *        "screenshot": "base64..."
 *      }
 *    - Store in history: "goToURL -> Navigated to https://news.ycombinator.com"
 *
 *    Iteration 2:
 *    -----------
 *    - Build re-prompt with:
 *      - ORIGINAL GOAL: What's on Hacker News front page?
 *      - CURRENT URL: https://news.ycombinator.com
 *      - Page content: [titles, links]
 *      - Actions so far: [goToURL]
 *    - Gemini call: "What's the NEXT single action?"
 *    - Gemini returns: { "function": "getText", "args": { "selector": ".storylink" } }
 *    - Execute: Extract text from story links
 *    - Result: Array of story titles
 *    - Store in history
 *
 *    Iteration 3:
 *    -----------
 *    - Build re-prompt with updated state
 *    - Gemini analyzes: "I have the story titles now, goal is achieved"
 *    - Gemini returns: {
 *        "function": "finish",
 *        "args": {
 *          "answer": "Here are the top stories on Hacker News:\n1. Show HN: I built...\n2. Ask HN: Best practices...\n..."
 *        }
 *      }
 *    - Loop breaks with final answer
 *    - Browser cleanup called
 *
 * 4. OUTPUT (in console):
 *    Task completed: Here are the top stories on Hacker News:
 *    1. Show HN: I built...
 *    2. Ask HN: Best practices...
 *    ...
 *
 * ============================================================================
 * KEY DESIGN DECISIONS:
 * ============================================================================
 *
 * 1. BACKGROUND EXECUTION:
 *    - handle_user_query spawns tokio task and returns immediately
 *    - Prevents blocking the UI while automation runs
 *    - User gets instant feedback that task started
 *
 * 2. STATEFUL LOOP:
 *    - Maintains action history throughout execution
 *    - Each Gemini call gets full context of what's been done
 *    - Enables self-correction and informed decision-making
 *
 * 3. RECURSIVE PROMPTING:
 *    - Initial prompt: "What's the FIRST action?"
 *    - Re-prompt: "Given current state + history, what's NEXT?"
 *    - Each prompt includes: goal, URL, DOM, links, inputs, history
 *    - Limits DOM text to 1000 chars to avoid token overflow
 *
 * 4. ERROR RECOVERY:
 *    - If Playwright step fails, ask Gemini for alternative
 *    - If recovery fails, abort gracefully with cleanup
 *    - Prevents infinite loops on broken actions
 *
 * 5. MAX ITERATIONS:
 *    - Hard limit of 10 iterations prevents infinite loops
 *    - Most tasks complete in 2-5 iterations
 *    - Complex multi-page flows may need adjustment
 *
 * 6. BROWSER LIFECYCLE:
 *    - Playwright maintains persistent browser session
 *    - Browser stays open across multiple actions (faster)
 *    - Explicit cleanup called at end of loop
 *
 * 7. FUNCTION CALL EXTRACTION:
 *    - Gemini can return text or function calls
 *    - extract_function_call handles both cases
 *    - Text containing "task complete" auto-converts to finish action
 *
 * ============================================================================
 * TESTING CHECKLIST:
 * ============================================================================
 *
 * Test Case 1: Simple Navigation
 * ------------------------------
 * Query: "Go to google.com"
 * Expected:
 *   - Iteration 1: goToURL(google.com)
 *   - Iteration 2: finish("Navigated to Google")
 *
 * Test Case 2: Information Extraction
 * ------------------------------------
 * Query: "What's on Hacker News front page?"
 * Expected:
 *   - Iteration 1: goToURL(news.ycombinator.com)
 *   - Iteration 2: getText or analyze DOM
 *   - Iteration 3: finish(list of stories)
 *
 * Test Case 3: Search Query
 * --------------------------
 * Query: "Search Google for 'rust async'"
 * Expected:
 *   - Iteration 1: goToURL(google.com)
 *   - Iteration 2: typeText(search input, "rust async")
 *   - Iteration 3: click(search button)
 *   - Iteration 4: getText(results)
 *   - Iteration 5: finish(search results summary)
 *
 * Test Case 4: Error Handling
 * ----------------------------
 * Query: "Click on .nonexistent-selector"
 * Expected:
 *   - Iteration 1: click fails
 *   - Recovery: Gemini suggests alternative
 *   - If no alternative: abort with cleanup
 *
 * ============================================================================
 * FUTURE ENHANCEMENTS:
 * ============================================================================
 *
 * 1. Real-time UI Updates:
 *    - Emit Tauri events on each iteration
 *    - Show live browser view in sidebar
 *    - Display action history in chat
 *
 * 2. Screenshot Integration:
 *    - Send screenshots to Gemini for visual reasoning
 *    - Use multimodal capabilities for better decisions
 *    - Display screenshots in chat log
 *
 * 3. Conversation History:
 *    - Support follow-up queries in same browser session
 *    - "Now click the first result" after search
 *    - Maintain context across multiple user messages
 *
 * 4. Advanced Actions:
 *    - Scroll, hover, drag-and-drop
 *    - File uploads, downloads
 *    - Multiple tabs/windows
 *
 * 5. Safety & Validation:
 *    - URL whitelist/blacklist
 *    - Action approval mode for sensitive operations
 *    - Rate limiting on Gemini calls
 *
 * ============================================================================
 */
