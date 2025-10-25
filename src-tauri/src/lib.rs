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

async fn call_gemini_api(query: String, api_key: String, model: String) -> Result<String, String> {
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
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, api_key
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

    let model = settings["model"]
        .as_str()
        .unwrap_or("gemini-1.5-flash")
        .to_string();

    // Check if API key is set
    if api_key.is_empty() {
        return Err(
            "API key not configured. Please go to Settings and enter your Gemini API key."
                .to_string(),
        );
    }

    // Try to call Gemini API with saved settings
    match call_gemini_api(query.clone(), api_key, model).await {
        Ok(response) => Ok(response),
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

#[tauri::command]
async fn save_settings(app: tauri::AppHandle, settings: String) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;

    let store = app
        .store("settings.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;

    store
        .set("app-settings", serde_json::json!(settings))
        .map_err(|e| format!("Failed to set settings: {}", e))?;

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
            save_settings,
            load_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
