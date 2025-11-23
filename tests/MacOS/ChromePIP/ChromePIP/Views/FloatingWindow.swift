import Cocoa
import AVFoundation

/// Floating window that displays the captured video stream
class FloatingPIPWindow: NSPanel {
    private let videoRenderer: VideoRenderer
    private let streamManager: StreamManager
    
    init(streamManager: StreamManager) {
        self.streamManager = streamManager
        self.videoRenderer = VideoRenderer()
        
        // Create window with initial size (320x180 for 16:9 aspect ratio)
        let initialFrame = NSRect(x: 100, y: 100, width: 640, height: 360)
        
        super.init(
            contentRect: initialFrame,
            styleMask: [.titled, .closable, .resizable],
            backing: .buffered,
            defer: false
        )
        
        // Configure window appearance
        self.title = "Chrome PIP"
        self.level = .floating // Always on top
        self.isOpaque = false
        self.backgroundColor = .black
        self.isMovableByWindowBackground = true
        
        // Setup video view
        setupVideoView()
        
        // Connect stream to renderer
        if let outputHandler = streamManager.getOutputHandler() {
            videoRenderer.connect(to: outputHandler)
        }
    }
    
    private func setupVideoView() {
        // Create a hosting view for the video layer
        let videoView = NSView(frame: self.contentView!.bounds)
        videoView.autoresizingMask = [.width, .height]
        videoView.wantsLayer = true
        
        // Add the video renderer's layer
        if let layer = videoRenderer.getLayer() {
            layer.frame = videoView.bounds
            layer.autoresizingMask = [.layerWidthSizable, .layerHeightSizable]
            videoView.layer?.addSublayer(layer)
        }
        
        self.contentView = videoView
    }
    
    override func close() {
        videoRenderer.disconnect()
        super.close()
    }
}
