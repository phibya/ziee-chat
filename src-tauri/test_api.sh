#!/bin/bash

echo "Testing streaming API..."
curl -X POST http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "test-llama",
    "messages": [{"role": "user", "content": "Hello"}],
    "max_tokens": 5,
    "temperature": 0.1,
    "stream": true
  }'