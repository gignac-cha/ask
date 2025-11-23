import SwiftUI

@main
struct ChromePIPApp: App {
    @StateObject private var appCoordinator = AppCoordinator()
    
    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(appCoordinator)
        }
    }
}

/// Central coordinator for managing app lifecycle and dependencies
class AppCoordinator: ObservableObject {
    @Published var isPIPActive = false
    @Published var statusMessage = "Ready"
    
    let permissionManager = PermissionManager()
    var browserController: BrowserController?
    var streamManager: StreamManager?
    var pipWindow: FloatingPIPWindow?
    
    func startPIP() {
        // Check permissions first
        guard permissionManager.hasScreenRecordingPermission() else {
            statusMessage = "Screen Recording permission required. Requesting..."
            permissionManager.requestScreenRecordingPermission()
            return
        }
        
        statusMessage = "Launching Chrome..."
        
        // Initialize browser controller
        browserController = BrowserController()
        
        Task {
            do {
                // Launch Chrome and get PID
                let chromePID = try await browserController?.launchChrome(
                    url: "https://www.google.com/?q=ask+browser",
                    width: 1920,
                    height: 1080
                )
                
                guard let pid = chromePID else {
                    await MainActor.run {
                        statusMessage = "Failed to launch Chrome"
                    }
                    return
                }
                
                await MainActor.run {
                    statusMessage = "Finding Chrome window..."
                }
                
                // Find Chrome window
                guard let chromeWindow = try await WindowFinder.findWindow(byProcessID: pid) else {
                    await MainActor.run {
                        statusMessage = "Chrome window not found"
                    }
                    return
                }
                
                await MainActor.run {
                    statusMessage = "Starting capture stream..."
                }
                
                // Create and start stream
                streamManager = StreamManager(targetWindow: chromeWindow)
                try await streamManager?.startCapture()
                
                await MainActor.run {
                    // Show PIP window with stream
                    showPIPWindow()
                    isPIPActive = true
                    statusMessage = "PIP Active"
                }
                
            } catch {
                await MainActor.run {
                    statusMessage = "Error: \(error.localizedDescription)"
                }
            }
        }
    }
    
    func stopPIP() {
        pipWindow?.close()
        pipWindow = nil
        
        Task {
            await streamManager?.stopCapture()
            streamManager = nil
            
            await browserController?.quitChrome()
            browserController = nil
            
            await MainActor.run {
                isPIPActive = false
                statusMessage = "Stopped"
            }
        }
    }
    
    private func showPIPWindow() {
        guard let streamManager = streamManager else { return }
        
        pipWindow = FloatingPIPWindow(streamManager: streamManager)
        pipWindow?.makeKeyAndOrderFront(nil)
    }
}
