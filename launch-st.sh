#!/bin/bash
# Launch Storyteller instance with separate data directory
export TAURI_IDENTIFIER_SUFFIX=".storyteller"
bun tauri dev
