import SwiftUI

struct ContentView: View {
    @EnvironmentObject var coordinator: AppCoordinator
    
    var body: some View {
        VStack(spacing: 20) {
            Text("Chrome PIP Controller")
                .font(.largeTitle)
                .fontWeight(.bold)
            
            Text(coordinator.statusMessage)
                .foregroundColor(.secondary)
            
            if !coordinator.isPIPActive {
                Button(action: {
                    coordinator.startPIP()
                }) {
                    Label("Start PIP", systemImage: "play.rectangle.fill")
                        .font(.title2)
                        .padding()
                        .background(Color.blue)
                        .foregroundColor(.white)
                        .cornerRadius(10)
                }
            } else {
                Button(action: {
                    coordinator.stopPIP()
                }) {
                    Label("Stop PIP", systemImage: "stop.rectangle.fill")
                        .font(.title2)
                        .padding()
                        .background(Color.red)
                        .foregroundColor(.white)
                        .cornerRadius(10)
                }
            }
            
            if !coordinator.permissionManager.hasScreenRecordingPermission() {
                VStack(spacing: 10) {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .font(.largeTitle)
                        .foregroundColor(.orange)
                    
                    Text("Screen Recording Permission Required")
                        .font(.headline)
                    
                    Text("This app needs permission to capture screen content. Please grant permission in System Settings > Privacy & Security > Screen Recording.")
                        .multilineTextAlignment(.center)
                        .foregroundColor(.secondary)
                        .padding(.horizontal)
                    
                    Button("Open System Settings") {
                        if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture") {
                            NSWorkspace.shared.open(url)
                        }
                    }
                    .padding(.top)
                }
                .padding()
            }
        }
        .frame(width: 400, height: 400)
        .padding()
    }
}

#Preview {
    ContentView()
        .environmentObject(AppCoordinator())
}
