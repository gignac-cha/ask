# Ask Engine

AI-powered browser automation desktop application built with Tauri, React, and TypeScript.

## Overview

Ask Engine is a conversational interface for web browsing. Instead of manually navigating websites, users can ask AI to perform web-based tasks and receive summarized results.

## Tech Stack

- **Application Framework:** Tauri (Rust backend)
- **UI:** React + TypeScript
- **AI/LLM:** Google Gemini (Function Calling/Tool Use)
- **Browser Automation:** Playwright

## Development

### Prerequisites

- Node.js (v18+)
- Rust
- System dependencies for Tauri (see [Tauri Prerequisites](https://tauri.app/start/prerequisites/))

### Getting Started

```bash
# Install dependencies
npm install

# Run development server
npm run tauri dev

# Build for production
npm run tauri build
```

## MVP Goal

Validate the ability to complete complex web-based tasks through AI chat interface.

Example: "Find and summarize 3 tutorials about using Tauri with Playwright."
