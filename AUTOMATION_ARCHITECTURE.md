# Ask Engine - Autonomous Automation Architecture

## System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                          FRONTEND (React/TypeScript)                │
│                                                                     │
│  ┌──────────────┐                                                  │
│  │  ChatPage    │                                                  │
│  │              │                                                  │
│  │ [User Input] │──────┐                                          │
│  └──────────────┘      │                                          │
│                        ▼                                          │
│                 invoke("handle_user_query", {query})              │
│                        │                                          │
└────────────────────────┼──────────────────────────────────────────┘
                         │ Tauri IPC
                         ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      BACKEND (Rust/Tauri)                          │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │ handle_user_query()                                          │ │
│  │  1. Load API key & model from settings                       │ │
│  │  2. Validate configuration                                   │ │
│  │  3. tokio::spawn(run_automation_loop)  ◄──── Background task │ │
│  │  4. Return "Task started..." immediately                     │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │ run_automation_loop()                   ┌──────────────────┐ │ │
│  │                                         │ ActionHistory[]  │ │ │
│  │  Loop (max 10 iterations):              │ - action         │ │ │
│  │  ┌────────────────────────────────┐     │ - args          │ │ │
│  │  │ 1. Call Gemini API             │     │ - result        │ │ │
│  │  │    ├─ Initial: Get first action│     └──────────────────┘ │ │
│  │  │    └─ Recursive: Get next step │                          │ │
│  │  │                                │                          │ │
│  │  │ 2. Extract function call       │                          │ │
│  │  │                                │                          │ │
│  │  │ 3. If "finish" → break         │                          │ │
│  │  │                                │                          │ │
│  │  │ 4. Execute via Playwright      │                          │ │
│  │  │                                │                          │ │
│  │  │ 5. Get DOM + screenshot        │                          │ │
│  │  │                                │                          │ │
│  │  │ 6. Update history              │                          │ │
│  │  │                                │                          │ │
│  │  │ 7. Build next prompt:          │                          │ │
│  │  │    - Original goal             │                          │ │
│  │  │    - Current URL               │                          │ │
│  │  │    - Page content (DOM)        │                          │ │
│  │  │    - Links & inputs            │                          │ │
│  │  │    - Action history            │                          │ │
│  │  │                                │                          │ │
│  │  │ 8. Continue loop               │                          │ │
│  │  └────────────────────────────────┘                          │ │
│  │                                                               │ │
│  │  Cleanup: Close browser                                      │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │ Helper Functions                                             │ │
│  │                                                               │ │
│  │ • execute_playwright_step(action_json)                       │ │
│  │   └─> Calls Node.js script via Command                      │ │
│  │                                                               │ │
│  │ • call_gemini_api(prompt, api_key, model)                    │ │
│  │   └─> HTTP POST to Gemini API                               │ │
│  │                                                               │ │
│  │ • extract_function_call(response)                            │ │
│  │   └─> Parse function call from Gemini response              │ │
│  │                                                               │ │
│  │ • create_next_step_prompt(goal, history, dom)                │ │
│  │   └─> Build context-aware prompt for next action            │ │
│  │                                                               │ │
│  │ • cleanup_playwright()                                       │ │
│  │   └─> Close browser and free resources                      │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
                         │                          │
                         ▼                          ▼
┌────────────────────────────────┐  ┌──────────────────────────────┐
│  PLAYWRIGHT (Node.js)          │  │  GEMINI API                  │
│                                │  │                              │
│  ┌──────────────────────────┐  │  │  ┌────────────────────────┐ │
│  │ playwright-step-         │  │  │  │ Function Calling       │ │
│  │   executor.mjs           │  │  │  │                        │ │
│  │                          │  │  │  │ Available tools:       │ │
│  │ Actions:                 │  │  │  │ • goToURL              │ │
│  │ • goToURL(url)           │  │  │  │ • click(selector)      │ │
│  │ • click(selector)        │  │  │  │ • typeText(...)        │ │
│  │ • typeText(sel, text)    │  │  │  │ • getText(selector)    │ │
│  │ • getText(selector)      │  │  │  │ • finish(answer)       │ │
│  │ • wait(ms)               │  │  │  └────────────────────────┘ │
│  │ • finish(answer)         │  │  │                              │
│  │                          │  │  │  Returns:                    │
│  │ Returns:                 │  │  │  • Function call with args   │
│  │ {                        │  │  │  • Or text response          │
│  │   success: bool          │  │  │                              │
│  │   dom: {...}             │  │  └──────────────────────────────┘
│  │   screenshot: base64     │  │
│  │   currentUrl: string     │  │
│  │   message: string        │  │
│  │ }                        │  │
│  └──────────────────────────┘  │
│                                │
│  Browser session maintained    │
│  across multiple actions       │
└────────────────────────────────┘
```

## Data Flow Example

### Query: "What's on Hacker News front page?"

```
1. User Input
   ↓
   "What's on Hacker News front page?"

2. Frontend → Backend (Tauri IPC)
   ↓
   invoke("handle_user_query", { query: "..." })

3. handle_user_query
   ↓
   • Validates settings
   • Spawns background task
   • Returns "Task started..."

4. run_automation_loop (Background)
   ↓

   [Iteration 1]
   ├─ Gemini API: "Determine FIRST action for: What's on Hacker News?"
   ├─ Response: { function: "goToURL", args: { url: "https://news.ycombinator.com" } }
   ├─ Execute: execute_playwright_step(action_json)
   ├─ Playwright: Navigate to URL
   ├─ Result: {
   │    success: true,
   │    dom: { url, title, mainText, links: [...], ... },
   │    screenshot: "base64..."
   │  }
   └─ History: ["goToURL → Navigated to https://news.ycombinator.com"]

   [Iteration 2]
   ├─ Build prompt with:
   │  • GOAL: What's on Hacker News front page?
   │  • URL: https://news.ycombinator.com
   │  • Content: [story titles from DOM]
   │  • History: [goToURL]
   ├─ Gemini API: "What's the NEXT action?"
   ├─ Response: {
   │    function: "finish",
   │    args: {
   │      answer: "Top stories on Hacker News:\n1. Show HN: I built...\n2. Ask HN: ..."
   │    }
   │  }
   ├─ Action is "finish" → Break loop
   └─ Final answer: "Top stories on Hacker News:..."

   [Cleanup]
   └─ cleanup_playwright() → Close browser

5. Output (Console)
   ↓
   "Task completed: Top stories on Hacker News:..."
```

## Prompt Engineering Strategy

### Initial Prompt (Iteration 1)
```
You are a browser automation assistant. The user wants: '{query}'

Determine the FIRST action to take to achieve this goal.

You have access to these browser actions:
- goToURL: Navigate to a URL
- click: Click an element by CSS selector
- typeText: Type text into an input by CSS selector
- getText: Extract text from an element by CSS selector

Respond with the first action to take. For example, if the user wants
information from a website, start by navigating to that website with goToURL.

Think about what website would be most helpful for this query and navigate there first.
```

### Recursive Prompt (Iteration 2+)
```
You are a browser automation assistant helping to achieve a specific goal.

ORIGINAL GOAL: {original_goal}

CURRENT PAGE STATE:
- URL: {current_url}
- Page Title: {page_title}
- Main Content Preview:
{main_text (limited to 1000 chars)}

- Available Links:
{top 10 links with text and href}

- Form Inputs:
{top 10 inputs with type, name, id}

ACTIONS COMPLETED SO FAR:
{numbered list of actions and results}

INSTRUCTIONS:
Based on the current page state and the original goal, determine the NEXT SINGLE action to take.

If you believe the goal has been achieved and you have gathered the necessary information,
return a "finish" action with the final answer.

Example finish action:
{"action": "finish", "args": {"answer": "Here is the information you requested: ..."}}

Otherwise, choose ONE of these actions:
- goToURL: Navigate to a URL
- click: Click an element (use CSS selector)
- typeText: Type into an input field (use CSS selector and text)
- getText: Extract text from an element (use CSS selector)

Think step-by-step about what action will best progress toward the goal.
Choose wisely and act decisively.
```

## Error Handling Flow

```
┌─────────────────────────┐
│ Execute Action          │
└───────────┬─────────────┘
            │
            ▼
    ┌───────────────┐
    │ Success?      │
    └───┬───────┬───┘
        │ Yes   │ No
        ▼       ▼
    Continue   ┌────────────────────────┐
    Loop       │ Error Recovery         │
               │                        │
               │ 1. Log error           │
               │ 2. Ask Gemini for      │
               │    alternative         │
               │ 3. If recovery works:  │
               │    Continue with new   │
               │    action              │
               │ 4. If recovery fails:  │
               │    Abort + cleanup     │
               └────────────────────────┘
```

## State Management

### ActionHistory Struct
```rust
ActionHistory {
    action: "goToURL",
    args: { "url": "https://example.com" },
    result: "Navigated to https://example.com"
}
```

### DOM Info Structure (from Playwright)
```json
{
  "url": "https://news.ycombinator.com",
  "title": "Hacker News",
  "mainText": "Full text content (limited to 3000 chars by Playwright)",
  "headings": ["Top", "New", "Best", ...],
  "links": [
    { "text": "Show HN: I built...", "href": "..." },
    { "text": "Ask HN: ...", "href": "..." }
  ],
  "inputs": [
    { "type": "text", "name": "q", "id": "search", "placeholder": "Search..." }
  ]
}
```

## Performance Characteristics

| Metric | Value | Notes |
|--------|-------|-------|
| **Initial Response Time** | < 100ms | Returns "Task started" immediately |
| **Browser Launch** | 1-2s | One-time cost per automation |
| **Per-iteration Time** | 2-3s | Gemini API call + action execution |
| **Typical Task Duration** | 4-15s | 2-5 iterations for most queries |
| **Max Task Duration** | 30s | 10 iterations × 3s average |
| **Memory Usage** | ~200MB | Chromium + minimal Rust overhead |
| **Token Usage** | 500-2000/iter | Depends on DOM size and history |

## Key Advantages of This Design

1. **Non-blocking**: UI remains responsive while automation runs
2. **Stateful**: AI has full context of what's been done
3. **Self-correcting**: Can recover from errors autonomously
4. **Transparent**: Comprehensive logging for debugging
5. **Extensible**: Easy to add new actions or tools
6. **Safe**: Max iterations prevent infinite loops
7. **Clean**: Automatic browser cleanup on completion

## Limitations & Future Work

1. **No UI Streaming**: Currently only logs to console
   - Future: Emit Tauri events for real-time updates

2. **Text-only Reasoning**: Doesn't use screenshots for decisions
   - Future: Send screenshots to Gemini for visual reasoning

3. **Single Session**: Each query starts fresh browser
   - Future: Support follow-up queries in same session

4. **Basic Actions**: Limited to goToURL, click, typeText, getText
   - Future: Add scroll, hover, file uploads, etc.

5. **No Safety Rails**: No URL filtering or action approval
   - Future: Whitelist/blacklist, approval mode

6. **Fixed Token Limits**: Hard-coded text truncation
   - Future: Smart summarization, dynamic limits
