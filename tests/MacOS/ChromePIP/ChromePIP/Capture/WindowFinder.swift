import Foundation
import ScreenCaptureKit

/// Finds specific SCWindow by process ID
class WindowFinder {
    
    enum WindowFinderError: Error {
        case windowNotFound
        case permissionDenied
    }
    
    /// Find a window by process ID
    /// - Parameter processID: The process ID to search for
    /// - Returns: The SCWindow object if found
    static func findWindow(byProcessID processID: Int32) async throws -> SCWindow? {
        // Get shareable content (all windows and displays)
        let content = try await SCShareableContent.current
        
        // Filter windows by process ID
        let matchingWindows = content.windows.filter { window in
            window.owningApplication?.processID == processID
        }
        
        // Return the first matching window (usually the main browser window)
        // We could add additional filtering by window title if needed
        guard let chromeWindow = matchingWindows.first else {
            throw WindowFinderError.windowNotFound
        }
        
        return chromeWindow
    }
    
    /// Find a window by application name and optional window title
    /// - Parameters:
    ///   - appName: Application name to search for
    ///   - windowTitle: Optional window title filter
    /// - Returns: The SCWindow object if found
    static func findWindow(byAppName appName: String, windowTitle: String? = nil) async throws -> SCWindow? {
        let content = try await SCShareableContent.current
        
        var matchingWindows = content.windows.filter { window in
            window.owningApplication?.applicationName == appName
        }
        
        // Further filter by window title if provided
        if let title = windowTitle {
            matchingWindows = matchingWindows.filter { window in
                window.title?.contains(title) ?? false
            }
        }
        
        guard let window = matchingWindows.first else {
            throw WindowFinderError.windowNotFound
        }
        
        return window
    }
}
