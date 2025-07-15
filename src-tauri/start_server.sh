#!/bin/bash

# Kill any existing model-server processes
pkill -f "model-server" 2>/dev/null || true

# Wait a moment for processes to terminate
sleep 2

# Start the model server
cd /Volumes/zData/Projects/react-test/src-tauri
./target/debug/model-server --model-path "/Users/bya/Downloads/2f5623ae-0f29-404e-a80f-d2f357d8817a" --architecture llama --port 8080 --model-id test-llama --app-data-dir "/Volumes/zData/Projects/react-test/.ziee"