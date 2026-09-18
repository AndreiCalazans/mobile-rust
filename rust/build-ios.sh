#!/usr/bin/env bash
# Build the Rust core for iOS (device + simulator), assemble an XCFramework,
# and regenerate the Swift bindings. All output is gitignored and rebuilt.
#
# Prereqs: rustup with iOS targets:
#   rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
set -euo pipefail

cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"

LIB=app_core
BUILD=../iosApp/Generated
XCF="$BUILD/AppCore.xcframework"

echo "==> Building Rust core for iOS targets"
cargo build --release --target aarch64-apple-ios
cargo build --release --target aarch64-apple-ios-sim
cargo build --release --target x86_64-apple-ios

echo "==> Generating Swift bindings"
mkdir -p "$BUILD"
rm -rf "$BUILD/swift"
cargo run --features bindgen --bin uniffi-bindgen -- generate \
  --library "target/aarch64-apple-ios/release/lib${LIB}.a" \
  --language swift \
  --out-dir "$BUILD/swift"

echo "==> Assembling universal simulator lib"
mkdir -p target/ios-sim-universal/release
lipo -create \
  "target/aarch64-apple-ios-sim/release/lib${LIB}.a" \
  "target/x86_64-apple-ios/release/lib${LIB}.a" \
  -output "target/ios-sim-universal/release/lib${LIB}.a"

echo "==> Building XCFramework"
# UniFFI emits a modulemap; rename it to module.modulemap for the framework.
HEADERS="$BUILD/headers"
rm -rf "$HEADERS" "$XCF"
mkdir -p "$HEADERS"
cp "$BUILD/swift/${LIB}FFI.h" "$HEADERS/"
cp "$BUILD/swift/${LIB}FFI.modulemap" "$HEADERS/module.modulemap"

xcodebuild -create-xcframework \
  -library "target/aarch64-apple-ios/release/lib${LIB}.a" -headers "$HEADERS" \
  -library "target/ios-sim-universal/release/lib${LIB}.a" -headers "$HEADERS" \
  -output "$XCF"

echo "==> Done. Swift in $BUILD/swift, framework at $XCF (both gitignored)."
