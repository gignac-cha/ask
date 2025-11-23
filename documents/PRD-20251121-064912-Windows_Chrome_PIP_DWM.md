# Product Requirements Document (PRD): Windows Chrome PIP (DWM Thumbnail)

**Date:** 2025-11-21
**Project Name:** Windows Chrome PIP (DWM Edition)
**Version:** 2.0
**Status:** Draft

---

## A. Information Gathering & Verification (Web Search)

To achieve the requirement of displaying a real 1920x1080 Chrome browser instance as a scaled-down PIP window, extensive technical research was conducted.

*   **Solution Comparison**:
    *   **WebView2**: Rejected. While efficient, it does not use the *actual* Chrome browser instance as requested for specific automation/extension needs.
    *   **SetParent (Window Embedding)**: Rejected. Embedding a complex multi-process application like Chrome into a WPF window via `SetParent` causes significant stability, rendering ("Airspace"), and input focus issues.
    *   **Screen Capture (BitBlt/DXGI)**: Rejected. High CPU usage for continuous streaming and lack of true real-time "live" feel.
    *   **DWM Thumbnail API**: **Selected**. The Windows Desktop Window Manager (DWM) provides a native API (`DwmRegisterThumbnail`) to render a live, hardware-accelerated reflection of one window onto another. This allows displaying the full 1920x1080 Chrome window scaled down to 320x180 with near-zero CPU overhead and perfect visual fidelity.

*   **Technical Feasibility**:
    *   Requires Windows 8 or later (Target: Windows 11).
    *   Requires `dwmapi.dll` P/Invoke calls.
    *   Requires Selenium WebDriver to control the "Source" Chrome window.

---

## B. Sub-agents Strategy (Parallel Execution)

To execute this complex integration efficiently, the following AI personas are defined:

*   **Sub-agent [Win32 Interop Specialist]**:
    *   **Responsibility**: Define and implement the unmanaged `dwmapi.dll` function calls.
    *   **Tasks**: Create the `DwmApi` static class with `DwmRegisterThumbnail`, `DwmUpdateThumbnailProperties`, and related structs (`RECT`, `DWM_THUMBNAIL_PROPERTIES`). Ensure correct P/Invoke signatures.
*   **Sub-agent [Browser Automation Engineer]**:
    *   **Responsibility**: Manage the Chrome browser lifecycle.
    *   **Tasks**: Implement Selenium ChromeDriver logic to launch Chrome off-screen (or hidden), navigate to the target URL, and reliably retrieve the correct Window Handle (HWND) for the browser process.
*   **Sub-agent [WPF UI Developer]**:
    *   **Responsibility**: Create the hosting UI.
    *   **Tasks**: Design the PIP container in XAML. Since DWM renders *directly* to the window surface, the UI container serves primarily as a placeholder and coordinate reference for the DWM rendering rectangle.

---

## C. Phased Implementation Plan

### Phase 1: Infrastructure & Dependencies
**Goal**: Prepare the project for Win32 API calls and Selenium automation.
**Deliverables**:
*   Install NuGet packages: `Selenium.WebDriver`, `Selenium.WebDriver.ChromeDriver`.
*   Create `DwmApi.cs` encapsulating all necessary P/Invoke definitions and constants.
*   Verify project targets a compatible .NET version (4.7.2+ or .NET 6+).

### Phase 2: Chrome Automation Core
**Goal**: Successfully launch and control a real Chrome instance.
**Deliverables**:
*   Implement `ChromeManager` class.
*   Launch Chrome with specific arguments (`--window-size=1920,1080`, `--window-position` off-screen to hide it from main view if desired).
*   Implement robust logic to find the specific `MainWindowHandle` of the launched Chrome instance (handling multi-process architecture).

### Phase 3: DWM Integration (The "PIP" Logic)
**Goal**: Link the Chrome window to the WPF application.
**Deliverables**:
*   Implement `RegisterThumbnail` logic: Call `DwmRegisterThumbnail` linking the Chrome HWND (Source) to the WPF Window HWND (Destination).
*   Implement `UpdateThumbnail` logic: Calculate the screen coordinates of the WPF PIP container and pass them to `DwmUpdateThumbnailProperties`.
*   Configure DWM properties: Set `Opacity`, `Visible`, and `SourceClientAreaOnly` (to strip Chrome's title bar if preferred).

### Phase 4: Interaction & Cleanup
**Goal**: Polish the user experience and ensure resource safety.
**Deliverables**:
*   **Toggle Logic**: Button to Start/Stop the PIP session.
*   **Window Management**: Ensure the thumbnail updates if the WPF window is moved or resized (handling `LocationChanged` events).
*   **Cleanup**: Critical implementation of `DwmUnregisterThumbnail` and `ChromeDriver.Quit()` on application exit to prevent zombie processes or memory leaks.

---

## D. Technical Specifications

*   **Source Resolution**: 1920 x 1080 (Logical).
*   **Target Resolution**: ~320 x 180 (Visual PIP).
*   **Scaling Method**: DWM Hardware Scaling (Bilinear/Anisotropic provided by GPU).
*   **Input Handling**: DWM Thumbnails are *visual only*. They do not accept clicks by default. (Requirement is display-only based on "PIP view").
*   **Window Handle Strategy**: Use `Process.MainWindowHandle` with retry logic or Selenium's `CurrentWindowHandle` mapped to a native HWND.

---
*End of PRD*
