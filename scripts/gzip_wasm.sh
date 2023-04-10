#!/bin/bash

# Set the hardcoded target directory
target_directory="../target/wasm32-unknown-unknown/release"

# Find all wasm files in the target directory and its subdirectories
find "$target_directory" -type f -name "*.wasm" | while read -r wasm_file; do
    # Compress the wasm file with gzip and save it with the same name and extension
    gzip -c "$wasm_file" > "${wasm_file}.gz"
done
