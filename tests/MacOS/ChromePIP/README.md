# Chrome PIP for macOS

A macOS native application that displays a live Chrome browser window as a floating Picture-in-Picture (PIP) window using ScreenCaptureKit.

## Features

- 🖥️ **Real-time Window Capture**: Uses ScreenCaptureKit for hardware-accelerated window capture
- 🌐 **Browser Automation**: Controls Chrome via WebDriver protocol
- 🎥 **Floating PIP Window**: Always-on-top window displaying the captured browser
- 🔒 **Permission Management**: Handles macOS Screen Recording permissions

## Requirements

- macOS 12.3 (Monterey) or later
- Xcode 14.0+
- Google Chrome
- ChromeDriver (install via `brew install chromedriver --cask`)

## Project Structure

```
ChromePIP/
├── ChromePIPApp.swift          # App entry point and coordinator
├── Info.plist                   # App configuration
├── Managers/
│   ├── PermissionManager.swift  # Screen recording permissions
│   └── BrowserController.swift  # Chrome automation
├── Capture/
│   ├── WindowFinder.swift       # Find Chrome window by PID
│   ├── StreamManager.swift      # ScreenCaptureKit stream
│   └── StreamOutputHandler.swift # Video frame handler
├── Rendering/
│   └── VideoRenderer.swift      # AVSampleBufferDisplayLayer wrapper
└── Views/
    ├── ContentView.swift        # Main control UI
    └── FloatingWindow.swift     # PIP window
```

## How It Works

1. **Permission Check**: Verifies Screen Recording permission
2. **Browser Launch**: Starts ChromeDriver and launches Chrome with specific window size
3. **Window Discovery**: Finds Chrome window using ScreenCaptureKit by process ID
4. **Stream Setup**: Creates SCStream to capture the Chrome window
5. **Rendering**: Displays captured frames in a floating window using AVSampleBufferDisplayLayer

## Building and Running

### Using Xcode (macOS only)

1. Open `ChromePIP.xcodeproj` in Xcode
2. Select your development team in Signing & Capabilities
3. Build and run (⌘R)

### First Run

On first launch, the app will request Screen Recording permission. You'll need to:

1. Grant permission in System Settings > Privacy & Security > Screen Recording
2. Restart the application after granting permission

## Technical Details

### ScreenCaptureKit Integration

- Uses `SCShareableContent` to enumerate windows
- Creates `SCContentFilter` for specific window capture
- Configures `SCStream` for 1920x1080 @ 30fps BGRA output
- Zero-copy rendering via `AVSampleBufferDisplayLayer`

### Browser Automation

- Communicates with ChromeDriver via HTTP (WebDriver protocol)
- Launches Chrome with custom window size and scale factor
- Retrieves process ID for window matching

### Performance

- Hardware-accelerated capture (GPU compositing)
- Low CPU overhead (ScreenCaptureKit is optimized for efficiency)
- Minimal memory footprint

## Troubleshooting

### ChromeDriver not found
Install via Homebrew:
```bash
brew install chromedriver --cask
```

### Permission denied
1. Go to System Settings > Privacy & Security > Screen Recording
2. Enable permission for ChromePIP
3. Restart the app

### Window not found
- Ensure Chrome is fully launched before capture starts
- Check that Chrome is running and visible (not minimized)

## Architecture Diagram

```
┌─────────────────┐
│  ContentView    │ (User clicks "Start PIP")
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────┐
│      AppCoordinator                 │
│  ┌──────────────────────────────┐  │
│  │ 1. PermissionManager          │  │
│  │    - Check/Request permission │  │
│  └──────────────────────────────┘  │
│  ┌──────────────────────────────┐  │
│  │ 2. BrowserController          │  │
│  │    - Launch Chrome            │  │
│  │    - Get PID                  │  │
│  └──────────────────────────────┘  │
│  ┌──────────────────────────────┐  │
│  │ 3. WindowFinder               │  │
│  │    - Find window by PID       │  │
│  └──────────────────────────────┘  │
│  ┌──────────────────────────────┐  │
│  │ 4. StreamManager              │  │
│  │    - Create SCStream          │  │
│  │    - Start capture            │  │
│  └──────────────────────────────┘  │
│  ┌──────────────────────────────┐  │
│  │ 5. FloatingPIPWindow          │  │
│  │    - Show PIP                 │  │
│  │    - Render frames            │  │
│  └──────────────────────────────┘  │
└─────────────────────────────────────┘
         │
         ▼
    ┌─────────┐
    │ Chrome  │ (Captured and displayed in PIP)
    └─────────┘
```

## License

MIT
