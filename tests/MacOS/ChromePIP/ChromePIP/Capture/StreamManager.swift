import Foundation
import ScreenCaptureKit
import AVFoundation

/// Manages the ScreenCaptureKit stream for capturing window content
class StreamManager: NSObject {
    private var stream: SCStream?
    private let targetWindow: SCWindow
    private var outputHandler: StreamOutputHandler?
    
    var isCapturing: Bool {
        stream != nil
    }
    
    init(targetWindow: SCWindow) {
        self.targetWindow = targetWindow
        super.init()
    }
    
    /// Start capturing the target window
    func startCapture() async throws {
        // Create content filter for the specific window
        let filter = SCContentFilter(desktopIndependentWindow: targetWindow)
        
        // Configure stream settings
        let configuration = SCStreamConfiguration()
        configuration.width = 1920
        configuration.height = 1080
        configuration.pixelFormat = kCVPixelFormatType_32BGRA
        configuration.minimumFrameInterval = CMTime(value: 1, timescale: 30) // 30 fps
        configuration.queueDepth = 5
        
        // Create output handler
        outputHandler = StreamOutputHandler()
        
        // Create and start stream
        stream = SCStream(filter: filter, configuration: configuration, delegate: self)
        
        try stream?.addStreamOutput(outputHandler!, type: .screen, sampleHandlerQueue: .global(qos: .userInteractive))
        try await stream?.startCapture()
    }
    
    /// Stop capturing
    func stopCapture() async {
        do {
            try await stream?.stopCapture()
        } catch {
            print("Error stopping capture: \(error)")
        }
        
        stream = nil
        outputHandler = nil
    }
    
    /// Get the output handler for connecting to renderer
    func getOutputHandler() -> StreamOutputHandler? {
        return outputHandler
    }
}

// MARK: - SCStreamDelegate

extension StreamManager: SCStreamDelegate {
    func stream(_ stream: SCStream, didStopWithError error: Error) {
        print("Stream stopped with error: \(error.localizedDescription)")
    }
}
