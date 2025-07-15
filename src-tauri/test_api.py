#!/usr/bin/env python3
import requests
import json

print("Testing streaming API...")

url = "http://127.0.0.1:8080/v1/chat/completions"
headers = {
    "Content-Type": "application/json"
}
data = {
    "model": "test-llama",
    "messages": [{"role": "user", "content": "Hello"}],
    "max_tokens": 5,
    "temperature": 0.1,
    "stream": True
}

try:
    response = requests.post(url, headers=headers, json=data, stream=True)
    print(f"Status Code: {response.status_code}")
    print("Response:")
    
    for chunk in response.iter_content(chunk_size=1024, decode_unicode=True):
        if chunk:
            print(chunk, end='')
    print()
    
except requests.exceptions.RequestException as e:
    print(f"Error: {e}")