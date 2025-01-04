#!/bin/bash

echo "Building for local architecture..."
cargo build --release

echo "Building for Raspberry Pi 3 architecture..."
cross build --release --target armv7-unknown-linux-gnueabihf
