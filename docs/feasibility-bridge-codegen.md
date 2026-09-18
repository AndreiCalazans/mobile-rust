# Feasibility: auto-generated + gitignored bridge code

Status: **Confirmed feasible.** Both the Android (JNI/Kotlin) bridge and the iOS
(Objective-C/Swift) bridge can be auto-generated at build time and kept out of
git. This document records the finding, the chosen tool, and the wiring that
makes it work.

## The requirement

The native apps hold **no business logic**. SwiftUI (iOS) and Jetpack Compose
(Android) render state and send intents. Everything from the *Use cases* layer
downward — use cases, repository, data sources, storage, API, transport — lives
in **Rust**. The line in the reference architecture that reads
`NATIVE · SWIFT / COMPOSE PER PLATFORM` moves **up above** *Use cases*.

The bridge code that lets Swift and Kotlin talk to Rust (JNI glue on Android,
Objective-C/Swift shims on iOS) must be:

1. **Auto-generated** — nobody writes it by hand.
2. **Gitignored** — it never enters version control; each build regenerates it.

## Decision: Mozilla UniFFI

We use **UniFFI** (proc-macro workflow, no `.udl` files). It is built by Mozilla
to share a Rust core between Firefox iOS (Swift) and Firefox Android (Kotlin),
so this is its exact use case.

| Need | UniFFI answer |
| --- | --- |
| Android JNI bridge | Auto-generated Kotlin + JNI glue |
| iOS Swift/ObjC bridge | Auto-generated Swift + C header + module map |
| Async logic | `async fn` maps to Kotlin `suspend` and Swift `async` |
| Tokio + reqwest | Runs futures on a background runtime; safe across the FFI |
| GraphQL | Rust returns typed structs; no wire format crosses the boundary |
| Generate at build time | `uniffi-bindgen generate --library <compiled lib>` |

Rejected alternatives:

- **cbindgen** — emits only C headers. All JNI and Swift glue is hand-written,
  and it has no async model. Fails requirement 1.
- **Diplomat** — no native async bridge (no `suspend` / `async` mapping), and a
  restricted type subset that fits GraphQL response structs poorly.

## Why gitignoring the generated code is safe

The bindings are a **pure function** of the compiled Rust library. Regenerating
them is deterministic, so the only risk — bindings drifting out of sync with the
Rust API — is removed by generating on **every** build rather than committing a
snapshot.

Trade-off accepted for this PoC: every build machine (dev + CI) needs the Rust
toolchain installed. That is acceptable because the whole point of the PoC is a
Rust-first mobile core.

### What is gitignored

```gitignore
# Rust build output
/rust/target/

# Auto-generated FFI bindings — regenerated on every build, never committed
/generated/
/androidApp/**/generated/
/iosApp/**/Generated/
```

## Build-time wiring

### Shared: embed the bindgen binary

`uniffi-bindgen` is embedded in the Rust crate behind an optional `bindgen`
feature so its CLI dependencies never link into the shipped `.a` / `.so`, and so
the bindgen version can never mismatch the runtime `uniffi` version.

```toml
[lib]
crate-type = ["cdylib", "staticlib", "rlib"]

[[bin]]
name = "uniffi-bindgen"
path = "src/bin/uniffi-bindgen.rs"
required-features = ["bindgen"]

[features]
bindgen = ["uniffi/cli"]
```

### Android (Gradle + cargo-ndk)

A Gradle task, run before Kotlin compilation:

1. `cargo ndk` builds `lib<core>.so` for each ABI.
2. `cargo run --features bindgen --bin uniffi-bindgen generate --library <.so>
   --language kotlin --out-dir <buildDir>/generated/uniffi`.
3. The generated dir is added as a Kotlin source set and is gitignored.

### iOS (Xcode build phase)

A "Run Script" phase, ordered before "Compile Sources":

1. `cargo build` for the iOS targets (device + simulator), assembled into an
   `.xcframework`.
2. `cargo run --features bindgen --bin uniffi-bindgen generate --library
   <lib>.a --language swift --out-dir <proj>/Generated`.
3. The generated `.swift`, the C header, and the modulemap land in a `Generated`
   folder that is gitignored.

## Conclusion

Requirement satisfied. UniFFI generates both bridges from the Rust core at build
time, supports async and the Tokio/reqwest/GraphQL stack the PoC needs, and its
output is safe to gitignore because each build regenerates it deterministically.
