#!/bin/bash

set -e

echo "Running cargo clippy with strict settings..."
cargo clippy -- --deny warnings

echo "✓ Clippy checks passed!"