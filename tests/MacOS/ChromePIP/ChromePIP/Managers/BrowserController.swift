import Foundation

/// Manages ChromeDriver service and browser control
class BrowserController {
    private var chromeDriverProcess: Process?
    private var chromeProcess: Process?
    private var chromeDriverPort: Int = 9515
    
    enum BrowserError: Error {
        case chromeDriverNotFound
        case chromeNotFound
        case launchFailed
        case communicationFailed
    }
    
    /// Launch Chrome browser with specified configuration
    /// - Parameters:
    ///   - url: URL to navigate to
    ///   - width: Window width
    ///   - height: Window height
    /// - Returns: Process ID of the Chrome browser
    func launchChrome(url: String, width: Int, height: Int) async throws -> Int32 {
        // Start ChromeDriver service first
        try await startChromeDriver()
        
        // Wait a bit for ChromeDriver to be ready
        try await Task.sleep(nanoseconds: 1_000_000_000) // 1 second
        
        // Launch Chrome via WebDriver protocol
        let session = try await createChromeSession(url: url, width: width, height: height)
        
        // Get Chrome process PID from the session
        // For now, we'll use a workaround to find Chrome process by name
        return try await findChromeProcess()
    }
    
    /// Quit Chrome browser
    func quitChrome() async {
        // Send quit command to ChromeDriver
        await deleteChromeSession()
        
        // Stop ChromeDriver
        chromeDriverProcess?.terminate()
        chromeDriverProcess = nil
    }
    
    // MARK: - Private Methods
    
    private func startChromeDriver() async throws {
        // Try to find chromedriver in common locations
        let chromedriverPaths = [
            "/usr/local/bin/chromedriver",
            "/opt/homebrew/bin/chromedriver",
            "/usr/bin/chromedriver"
        ]
        
        guard let chromedriverPath = chromedriverPaths.first(where: { FileManager.default.fileExists(atPath: $0) }) else {
            throw BrowserError.chromeDriverNotFound
        }
        
        let process = Process()
        process.executableURL = URL(fileURLWithPath: chromedriverPath)
        process.arguments = ["--port=\(chromeDriverPort)"]
        
        try process.run()
        chromeDriverProcess = process
    }
    
    private func createChromeSession(url: String, width: Int, height: Int) async throws -> String {
        // Create a new Chrome session via WebDriver protocol
        let endpoint = "http://localhost:\(chromeDriverPort)/session"
        
        let capabilities: [String: Any] = [
            "capabilities": [
                "alwaysMatch": [
                    "browserName": "chrome",
                    "goog:chromeOptions": [
                        "args": [
                            "--window-size=\(width),\(height)",
                            "--force-device-scale-factor=1"
                        ]
                    ]
                ]
            ]
        ]
        
        guard let url = URL(string: endpoint),
              let jsonData = try? JSONSerialization.data(withJSONObject: capabilities) else {
            throw BrowserError.communicationFailed
        }
        
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = jsonData
        
        let (data, response) = try await URLSession.shared.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 else {
            throw BrowserError.launchFailed
        }
        
        // Parse session ID from response
        if let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
           let value = json["value"] as? [String: Any],
           let sessionId = value["sessionId"] as? String {
            
            // Navigate to URL
            try await navigateToURL(sessionId: sessionId, url: url)
            
            return sessionId
        }
        
        throw BrowserError.communicationFailed
    }
    
    private func navigateToURL(sessionId: String, url: String) async throws {
        let endpoint = "http://localhost:\(chromeDriverPort)/session/\(sessionId)/url"
        
        let payload: [String: Any] = ["url": url]
        
        guard let endpointURL = URL(string: endpoint),
              let jsonData = try? JSONSerialization.data(withJSONObject: payload) else {
            throw BrowserError.communicationFailed
        }
        
        var request = URLRequest(url: endpointURL)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = jsonData
        
        _ = try await URLSession.shared.data(for: request)
    }
    
    private func deleteChromeSession() async {
        // Implementation for closing the session
        // This would involve sending DELETE to /session/{sessionId}
    }
    
    private func findChromeProcess() async throws -> Int32 {
        // Use shell command to find Chrome process
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/bin/sh")
        task.arguments = ["-c", "pgrep -n 'Google Chrome'"]
        
        let pipe = Pipe()
        task.standardOutput = pipe
        
        try task.run()
        task.waitUntilExit()
        
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        if let output = String(data: data, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines),
           let pid = Int32(output) {
            return pid
        }
        
        throw BrowserError.launchFailed
    }
}
