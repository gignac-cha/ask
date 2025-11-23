import Foundation
import AVFoundation

/// Manages video rendering using AVSampleBufferDisplayLayer
class VideoRenderer {
    private let displayLayer: AVSampleBufferDisplayLayer
    
    init() {
        displayLayer = AVSampleBufferDisplayLayer()
        displayLayer.videoGravity = .resizeAspect
        displayLayer.backgroundColor = CGColor.black
    }
    
    /// Get the CALayer for adding to view hierarchy
    func getLayer() -> CALayer {
        return displayLayer
    }
    
    /// Connect to a stream output handler
    func connect(to outputHandler: StreamOutputHandler) {
        outputHandler.connect(to: displayLayer)
    }
    
    /// Disconnect from stream
    func disconnect() {
        // Flush any pending samples
        displayLayer.flush()
    }
    
    /// Check if renderer is ready to display
    var isReadyForDisplay: Bool {
        return displayLayer.isReadyForMoreMediaData
    }
}
