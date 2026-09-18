import ProjectDescription

// -----------------------------------------------------------------------------
// Tuist project for the Rust-core PoC iOS app.
//
// The app holds no logic. It links a single AppCore.xcframework (the compiled
// Rust static lib + C header + modulemap) and compiles the UniFFI-generated
// Swift wrapper. Both are produced by ../rust/build-ios.sh and are gitignored.
//
// A pre-build script runs build-ios.sh before "Compile Sources", so the
// framework and the generated Swift always exist and are always fresh.
// -----------------------------------------------------------------------------

// Build the Rust core + regenerate the (gitignored) Swift bindings + XCFramework.
let buildRustScript = """
cd "$SRCROOT/../rust"
export PATH="$HOME/.cargo/bin:$PATH"
./build-ios.sh
"""

let project = Project(
    name: "MobileRust",
    targets: [
        .target(
            name: "MobileRust",
            destinations: .iOS,
            product: .app,
            bundleId: "com.example.mobilerust",
            deploymentTargets: .iOS("16.0"),
            infoPlist: .extendingDefault(with: [
                "UILaunchScreen": [:],
            ]),
            sources: [
                "MobileRust/**",
                // UniFFI-generated Swift wrapper (gitignored, built by build-ios.sh).
                "Generated/swift/**",
            ],
            scripts: [
                .pre(
                    script: buildRustScript,
                    name: "Build Rust core + generate bindings",
                    basedOnDependencyAnalysis: false
                ),
            ],
            dependencies: [
                // The Rust core as a prebuilt framework (gitignored).
                .xcframework(path: "Generated/AppCore.xcframework"),
            ],
            settings: .settings(base: [
                "SWIFT_VERSION": "5.0",
                "TARGETED_DEVICE_FAMILY": "1,2",
                "CODE_SIGNING_ALLOWED": "NO",
                "ENABLE_USER_SCRIPT_SANDBOXING": "NO",
            ])
        ),
    ]
)
