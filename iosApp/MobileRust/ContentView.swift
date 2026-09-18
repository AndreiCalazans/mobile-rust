import SwiftUI
// The UniFFI-generated Swift wrapper (Generated/swift/**) is compiled directly
// into this target, so its types (AppCore, SessionState, ...) are in scope with
// no import. It links the C module app_coreFFI from the xcframework internally.

// The view model owns the single Rust AppCore. All logic is in Rust; this only
// dispatches intents and projects state.
@MainActor
final class AppModel: ObservableObject {
    private let core = AppCore()  // Rust-backed handle (ARC-managed)

    @Published var session: SessionState
    @Published var words: [WordEntry] = []
    @Published var bitcoin: AssetPrice?
    @Published var status: String = ""

    init() { session = core.currentSession() }

    func login()  { session = core.login(displayName: "Ada Lovelace") }
    func logout() { core.logout(); session = core.currentSession() }

    func loadWords() {
        status = "loading words…"
        Task {
            words = await core.defineWords(words: ["serendipity", "rust", "mobile"])
            status = ""
        }
    }

    func loadBitcoin() {
        status = "loading BTC…"
        Task {
            do { bitcoin = try await core.fetchBitcoin(); status = "" }
            catch { status = "BTC error: \(error)" }
        }
    }
}

struct ContentView: View {
    @StateObject private var model = AppModel()

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                Text("Rust-core PoC").font(.title2).bold()

                switch model.session {
                case let .active(user, _):
                    Text("Signed in: \(user.displayName)")
                    Button("Log out") { model.logout() }
                case .loggedOut:
                    Text("Signed out")
                    Button("Fake login") { model.login() }
                }

                Button("Fetch words (REST)")  { model.loadWords() }
                Button("Fetch BTC (GraphQL)") { model.loadBitcoin() }

                if !model.status.isEmpty { Text(model.status) }

                if let btc = model.bitcoin {
                    Text("\(btc.symbol) (\(btc.name)) = $\(btc.priceUsd)  day \(btc.changeDayPercent)%")
                }

                ForEach(model.words, id: \.word) { w in
                    Text("• \(w.word) \(w.phonetic ?? "") — \(w.definitions.first?.meaning ?? "")")
                }
            }
            .padding(24)
        }
    }
}
