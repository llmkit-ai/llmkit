#!/bin/bash

# LLMKit installation script
# This script installs the 'llmkit' command globally

set -e

echo "📦 Installing LLMKit..."

# Build the llmkit binary
echo "🔨 Building llmkit binary..."
cd backend
cargo build --bin llmkit --release

# Create symlink to cargo bin directory
echo "🔗 Creating command symlink..."
cargo install --path . --bin llmkit --force

echo "✅ LLMKit installed successfully!"
echo ""
echo "Run 'llmkit start' to start the application"