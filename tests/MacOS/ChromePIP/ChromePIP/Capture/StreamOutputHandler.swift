import Foundation
import ScreenCaptureKit
import AVFoundation

/// Handles video frame output from SCStream
class StreamOutputHandler: NSObject, SCStreamOutput {
    private var sampleBufferDisplayLayer: AVSampleBufferDisplayLayer?
    
    /// Connect to a display layer for rendering
    func connect(to displayLayer: AVSampleBufferDisplayLayer) {
        self.sampleBufferDisplayLayer = displayLayer
    }
    
    /// Disconnect from display layer
    func disconnect() {
        sampleBufferDisplayLayer = nil
    }
    
    // MARK: - SCStreamOutput
    
    func stream(_ stream: SCStream, didOutputSampleBuffer sampleBuffer: CMSampleBuffer, of type: SCStreamOutputType) {
        // Ensure we're handling screen content
        guard type == .screen else { return }
        
        // Ensure the sample buffer is valid
        guard sampleBuffer.isValid else { return }
        
        // Forward to display layer if connected
        if let displayLayer = sampleBufferDisplayLayer {
            // Enqueue on main thread as AVSampleBufferDisplayLayer requires it
            DispatchQueue.main.async {
                displayLayer.enqueue(sampleBuffer)
            }
        }
    }
}
