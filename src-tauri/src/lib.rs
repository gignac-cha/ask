use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;

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
}

#[derive(Serialize, Deserialize, Debug)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum ResponsePart {
    Text { text: String },
    FunctionCall { function_call: FunctionCall },
}

#[derive(Serialize, Deserialize, Debug)]
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
        ],
    }]
}

async fn call_gemini_api(query: String) -> Result<String, String> {
    let api_key = env::var("GEMINI_API_KEY").map_err(|_| {
        "GEMINI_API_KEY environment variable not set. Set it to use Gemini AI.".to_string()
    })?;

    let tools = create_browser_tools();

    let request_body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: format!(
                    "You are a browser automation assistant. The user wants: '{}'. \
                     Analyze this request and determine which browser actions to perform. \
                     You have access to these functions: goToURL, click, getText, typeText. \
                     Plan the steps needed to fulfill this request.",
                    query
                ),
            }],
        }],
        tools,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            api_key
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

    let gemini_response: GeminiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Gemini response: {}", e))?;

    // Extract function calls or text response
    if let Some(candidate) = gemini_response.candidates.first() {
        let mut result = String::new();

        for part in &candidate.content.parts {
            match part {
                ResponsePart::Text { text } => {
                    result.push_str(text);
                    result.push('\n');
                }
                ResponsePart::FunctionCall { function_call } => {
                    let action_json = serde_json::json!({
                        "function": function_call.name,
                        "args": function_call.args
                    });
                    result.push_str(&format!("Action: {}\n", action_json));
                }
            }
        }

        Ok(result.trim().to_string())
    } else {
        Err("No response from Gemini".to_string())
    }
}

#[tauri::command]
async fn handle_user_query(query: String) -> Result<String, String> {
    println!("Received query: {}", query);

    // Try to call Gemini API
    match call_gemini_api(query.clone()).await {
        Ok(response) => Ok(response),
        Err(e) if e.contains("GEMINI_API_KEY") => {
            // If API key is not set, return a mock response
            Ok(format!(
                "Mock response (Set GEMINI_API_KEY to use real AI):\n\n\
                 For query: '{}'\n\n\
                 Planned actions:\n\
                 1. goToURL(\"https://www.google.com\")\n\
                 2. typeText(\"input[name='q']\", \"{}\")\n\
                 3. click(\"input[type='submit']\")\n\
                 4. getText(\".search-result\")",
                query, query
            ))
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
async fn execute_browser_actions(actions: String) -> Result<String, String> {
    println!("Executing browser actions: {}", actions);

    // Get the project root directory (assuming we're in src-tauri/target/...)
    let current_dir = env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

    // Look for scripts directory relative to current location
    let script_path = current_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("scripts").join("playwright-executor.mjs"))
        .ok_or_else(|| "Could not find scripts directory".to_string())?;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            handle_user_query,
            execute_browser_actions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
