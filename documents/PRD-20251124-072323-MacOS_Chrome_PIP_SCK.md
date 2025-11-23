# Product Requirements Document (PRD): macOS Chrome PIP (ScreenCaptureKit)

**Date:** 2025-11-24
**Project Name:** macOS Chrome PIP (SCK Edition)
**Version:** 1.0
**Status:** Draft

---

## A. Information Gathering & Verification (Web Search)

To replicate the "Live Browser PIP" functionality on macOS (previously implemented on Windows via DWM), specific macOS native technologies were researched and selected to ensure high performance and low resource usage.

*   **Technology Selection & Verification**:
    *   **ScreenCaptureKit (SCK)**: Verified as the modern replacement for `CGWindowListCreateImage`. Available on macOS 12.3+. It allows capturing specific windows with GPU acceleration and zero-copy semantics, making it ideal for a "Live PIP" view.
    *   **Selenium WebDriver**: Verified compatibility with macOS. Requires `chromedriver` binary. The logic for launching Chrome with specific flags (`--window-size`) remains consistent with the Windows implementation.
    *   **Window Identification**: Verified that Selenium `CurrentWindowHandle` does not directly map to macOS `CGWindowID`. A bridging strategy using `CGWindowListCopyWindowInfo` or `SCShareableContent` (matching by Process ID or Window Title) is required to link the automation controller to the visual capturer.
    *   **Permissions**: Noted that macOS requires explicit "Screen Recording" permission for the application to capture other windows. The app must handle the permission request flow gracefully.

---

## B. Sub-agents Strategy (Parallel Execution)

To ensure a robust and native implementation, the development will be divided among specialized AI personas:

*   **Sub-agent [macOS System Engineer]**:
    *   **Responsibility**: Handle low-level OS interactions and permissions.
    *   **Tasks**: Implement `ScreenCaptureKit` logic to query shareable content and create an `SCStream`. Handle `Info.plist` configurations for permissions (Screen Recording). Implement the logic to find the correct `WindowID` given a Process ID (PID) from Selenium.
*   **Sub-agent [SwiftUI/UX Designer]**:
    *   **Responsibility**: Create the Floating PIP Window.
    *   **Tasks**: Design a SwiftUI View that hosts the video stream. Implement the `NSPanel` or `NSWindow` subclass to ensure the window stays "Always on Top" (Floating), has no title bar, and supports resizing/dragging.
*   **Sub-agent [Automation Specialist]**:
    *   **Responsibility**: Port and adapt the Selenium logic.
    *   **Tasks**: Configure `Selenium.WebDriver` for macOS. Manage the `chromedriver` service lifecycle. Ensure Chrome launches in a way that is capture-friendly (e.g., correct initial resolution).

---

## C. Phased Implementation Plan

### Phase 1: Project Initialization & Entitlements
**Goal**: Set up a secure macOS Sandbox-friendly environment (or non-sandboxed if required for Selenium) capable of screen recording.
**Deliverables**:
*   Initialize a new Xcode Project (Swift/SwiftUI) in `tests/MacOS`.
*   Configure `Signing & Capabilities`: Add "Hardened Runtime" if distributing, or manage local signing.
*   **Critical**: Implement a check for `CGPreflightScreenCaptureAccess()` and request Screen Recording permissions if not granted.
*   Setup dependency management (Swift Package Manager) for Selenium (if available) or manual inclusion of WebDriver binaries.

### Phase 2: Browser Automation Core
**Goal**: Launch Chrome and control it programmatically.
**Deliverables**:
*   Implement `ChromeAutomationManager` class.
*   Logic to locate the `chromedriver` binary bundled with the app or in a known path.
*   Launch Chrome with arguments: `--window-size=1920,1080`, `--force-device-scale-factor=1` (to ensure 1:1 pixel mapping if needed).
*   Retrieve the `Process ID (PID)` of the launched Chrome instance.

### Phase 3: Target Discovery & Capture (The "SCK" Logic)
**Goal**: Find the visual window corresponding to the automated browser and start the video stream.
**Deliverables**:
*   Implement `WindowFinder`: Use `SCShareableContent.current` to list windows. Filter by the Chrome PID obtained in Phase 2 to find the specific `SCWindow` object.
*   Implement `StreamManager`: Create an `SCStream` targeting the identified `SCWindow`.
*   Configure `SCStreamConfiguration`: Set width/height (1920x1080), pixel format (BGRA), and frame rate (e.g., 30 or 60 fps).

### Phase 4: Rendering & UI Integration
**Goal**: Display the captured stream in a floating window.
**Deliverables**:
*   Implement a `StreamOutputHandler` implementing `SCStreamOutput`. Receive `CMSampleBuffer` frames.
*   **Rendering Layer**:
    *   *Option A (Easier)*: Use `AVSampleBufferDisplayLayer` (AVFoundation) wrapped in an `NSViewRepresentable`.
    *   *Option B (Performance)*: Use `MTKView` (Metal) to render the textures directly.
*   **PIP Window**: Create a `FloatingWindow` (subclass of `NSWindow`) that:
    *   Has `.floating` level.
    *   Is borderless/transparent background.
    *   Hosts the rendering view.

### Phase 5: Interaction & Lifecycle Management
**Goal**: Ensure the app behaves like a polished utility.
**Deliverables**:
*   **Sync Logic**: If the user resizes the PIP window, adjust the `SCStreamConfiguration` scale if necessary (though usually, we just scale the view).
*   **Cleanup**: Ensure `SCStream.stopCapture` is called and `driver.quit()` is executed when the app closes to prevent resource leaks.
*   **Error Handling**: Handle cases where the Chrome window is minimized (SCK might stop sending frames) or closed.

---

## D. Technical Specifications

*   **Source Resolution**: 1920 x 1080 (Logical).
*   **Target View**: Resizable Floating Window (maintaining aspect ratio).
*   **Capture API**: `ScreenCaptureKit` (Requires macOS 12.3+).
*   **Rendering API**: `AVSampleBufferDisplayLayer` (Recommended for video stream efficiency) or `Metal`.
*   **Automation**: Selenium WebDriver (communicating via local localhost port to `chromedriver`).
*   **OS Requirement**: macOS Monterey 12.3 or later.

---
*End of PRD*
