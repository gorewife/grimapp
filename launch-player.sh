#!/bin/bash
# Launch Player instance with separate data directory
export TAURI_IDENTIFIER_SUFFIX=".player"
bun tauri dev
