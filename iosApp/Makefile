# iOS project bootstrap.
#
# Tuist validates every path at generation time, BEFORE any build script runs.
# The app links Generated/AppCore.xcframework and compiles Generated/swift/**,
# both produced by ../rust/build-ios.sh. So the Rust artifacts must exist before
# `tuist generate`. This Makefile enforces that order.
#
# Usage:
#   make setup     # build Rust iOS artifacts, then generate the Xcode project
#   make generate  # regenerate the project only (artifacts must already exist)
#   make open      # setup + open in Xcode

.PHONY: setup generate open rust clean

rust:
	cd ../rust && ./build-ios.sh

# First-time / clean generation: build the Rust core, then generate the project.
setup: rust generate

generate:
	tuist generate

open: setup
	tuist generate --open

clean:
	rm -rf Generated
	tuist clean
