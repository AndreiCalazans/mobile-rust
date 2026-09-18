#!/usr/bin/env bash
# Build the Rust core for Android ABIs and regenerate the Kotlin bindings.
# Output (.so libs + generated Kotlin) is gitignored; this runs on every build.
#
# Prereqs: rustup, cargo-ndk, Android NDK. Install ABIs with:
#   rustup target add aarch64-linux-android armv7-linux-androideabi \
#                     x86_64-linux-android i686-linux-android
#   cargo install cargo-ndk
set -euo pipefail

cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"

JNI_LIBS="../androidApp/app/src/main/jniLibs"
KOTLIN_OUT="../androidApp/app/build/generated/uniffi"

echo "==> Building Rust core for Android ABIs"
cargo ndk \
  -t arm64-v8a -t armeabi-v7a -t x86_64 -t x86 \
  -o "$JNI_LIBS" \
  build --release

echo "==> Generating Kotlin bindings"
rm -rf "$KOTLIN_OUT"
cargo run --features bindgen --bin uniffi-bindgen -- generate \
  --library "$JNI_LIBS/arm64-v8a/libapp_core.so" \
  --language kotlin \
  --out-dir "$KOTLIN_OUT"

echo "==> Done. Libs in $JNI_LIBS, Kotlin in $KOTLIN_OUT (both gitignored)."
