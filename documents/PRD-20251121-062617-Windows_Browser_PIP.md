# Product Requirements Document (PRD): Windows Browser PIP

**Date:** 2025-11-21
**Project Name:** Windows Browser PIP
**Version:** 1.0
**Status:** Draft

---

## A. Information Gathering & Verification (Web Search)

Before defining the requirements, technical feasibility was validated through web research.

*   **Visual Studio 2026 Context**: Confirmed that Visual Studio 2026 is the latest stable release (GA Nov 2025) with .NET 10 support. The project will target .NET 10 (or latest supported by the user's specific environment if restricted to Framework, but assuming modern .NET based on VS version) to leverage the latest WebView2 performance improvements.
*   **WebView2 Scaling**:
    *   **Challenge**: Displaying a 1920x1080 logical browser surface within a ~320x240 visual container.
    *   **Solution**: The standard WPF `Viewbox` often causes rendering artifacts with WebView2. The recommended approach is using a `ScaleTransform` on the WebView2 control's `RenderTransform`.
    *   **Calculation**: To fit 1920 width into 320 width, a scale factor of approx `0.166` is required.
    *   **Aspect Ratio Note**: 1920x1080 is a 16:9 ratio. 320x240 is a 4:3 ratio. Directly fitting 16:9 into 4:3 will result in letterboxing (black bars top/bottom) or cropping. The implementation will prioritize fitting the width (320px) which results in a height of ~180px, centered within the 240px container, or adjusting the container to 320x180 to match.

---

## B. Sub-agents Strategy (Parallel Execution)

To maximize efficiency, the following AI personas (Sub-agents) are defined for parallel execution:

*   **Sub-agent [UI Architect]**:
    *   **Responsibility**: Design the XAML layout for `MainWindow`.
    *   **Tasks**: Define the Grid layout, position the central "Open PIP" button, and define the `Border` container for the PIP view in the bottom-left corner. Apply `RenderTransform` properties to the WebView2 placeholder.
*   **Sub-agent [Core Logic Dev]**:
    *   **Responsibility**: Implement C# code-behind and WebView2 initialization.
    *   **Tasks**: Handle button click events, initialize WebView2 with specific arguments (if needed for high DPI), navigate to the target URL, and manage the visibility toggle of the PIP window.
*   **Sub-agent [QA Specialist]**:
    *   **Responsibility**: Define test cases for resolution and interaction.
    *   **Tasks**: Verify the browser reports a viewport of 1920x1080 (via JS console check) while physically occupying ~320x240 pixels on screen.

---

## C. Phased Implementation Plan

### Phase 1: Project Initialization & Configuration
**Goal**: Establish a stable build environment with necessary dependencies.
**Deliverables**:
*   Solution created in `tests/Windows`.
*   NuGet package `Microsoft.Web.WebView2` installed.
*   `app.manifest` configured for Per-Monitor DPI awareness (critical for correct WebView2 rendering).

### Phase 2: UI Layout & PIP Container Setup
**Goal**: Create the visual structure without active logic.
**Deliverables**:
*   **Main Window**: Standard WPF window (800x600 or default).
*   **Central Button**: Labeled "Launch Browser PIP", centered in the grid.
*   **PIP Container**: A `Grid` or `Border` fixed to the bottom-left (VerticalAlignment="Bottom", HorizontalAlignment="Left").
*   **WebView2 Control**: Placed inside the PIP container.
    *   *Crucial Setup*: The WebView2 control must be sized explicitly to **1920x1080** in XAML (Width="1920", Height="1080").
    *   *Scaling*: Apply a `ScaleTransform` of `0.166` (X/Y) to shrink it visually to ~320px width.
    *   *Origin*: Set `RenderTransformOrigin` to "0,1" (Bottom-Left) or adjust margins to align it correctly within the visual container.

### Phase 3: Interaction Logic & Navigation
**Goal**: Make the application functional.
**Deliverables**:
*   **Button Event**: On click, toggle the visibility of the PIP Container from `Collapsed` to `Visible`.
*   **Navigation**: Trigger `WebView2.CoreWebView2.Navigate("https://www.google.com/?q=ask+browser")`.
*   **Lifecycle Management**: Ensure WebView2 is initialized async before navigation.

### Phase 4: Verification & Refinement
**Goal**: Ensure requirements are met and UX is smooth.
**Deliverables**:
*   **Resolution Check**: Inject JavaScript `alert(window.innerWidth + 'x' + window.innerHeight)` to confirm the site "thinks" it is 1920x1080.
*   **Visual Check**: Ensure the text is tiny (scaled down) but the layout matches a full desktop desktop browser, not a mobile view.
*   **Aspect Ratio Handling**: Confirm that the 16:9 content is centered or aligned as preferred within the 4:3 space.

---

## D. Technical Specifications & Constraints

*   **Target Framework**: .NET 10 (via Visual Studio 2026) or .NET Framework 4.8 (if legacy required). *Recommendation: Use .NET 6+ for better WebView2 support.*
*   **Library**: `Microsoft.Web.WebView2` (Latest Stable).
*   **Resolution Logic**:
    *   **Logical Size**: 1920px x 1080px.
    *   **Visual Size**: ~320px x 180px (maintaining aspect ratio).
    *   **Implementation Detail**:
        ```text
        <WebView2 Width="1920" Height="1080">
            <WebView2.RenderTransform>
                <ScaleTransform ScaleX="0.1666" ScaleY="0.1666" />
            </WebView2.RenderTransform>
        </WebView2>
        ```
        *Note: The container must clip bounds if the scaled height exceeds the desired area, though 180px fits within 240px.*

---
*End of PRD*
