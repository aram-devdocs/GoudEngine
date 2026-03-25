import SwiftUI
import GoudEngine

@main
struct MobileTemplateApp: App {
    init() {
        _ = EngineConfig()
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}
