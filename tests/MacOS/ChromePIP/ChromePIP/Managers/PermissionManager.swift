import Foundation
import CoreGraphics

/// Manages screen recording permission for macOS
class PermissionManager {
    
    /// Check if the app has screen recording permission
    func hasScreenRecordingPermission() -> Bool {
        // CGPreflightScreenCaptureAccess checks permission without triggering a prompt
        return CGPreflightScreenCaptureAccess()
    }
    
    /// Request screen recording permission from the user
    /// This will trigger the system permission dialog
    func requestScreenRecordingPermission() {
        // CGRequestScreenCaptureAccess triggers the permission dialog
        // Returns true if permission is already granted, false if dialog is shown
        let granted = CGRequestScreenCaptureAccess()
        
        if !granted {
            print("Screen recording permission not yet granted. User must approve in System Settings.")
        }
    }
    
    /// Check and request permission if needed
    func ensurePermission() -> Bool {
        if hasScreenRecordingPermission() {
            return true
        }
        
        requestScreenRecordingPermission()
        return false
    }
}
