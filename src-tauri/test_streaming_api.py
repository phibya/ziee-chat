#!/usr/bin/env python3
"""
Test script to verify streaming API functionality for the model server.
This script tests multiple consecutive streaming requests to identify any issues.
"""

import requests
import json
import time
import sys

def test_streaming_request(request_num):
    """Test a single streaming request."""
    url = "http://127.0.0.1:8080/v1/chat/completions"
    headers = {"Content-Type": "application/json"}
    data = {
        "model": "test-llama",
        "messages": [{"role": "user", "content": "Hello"}],
        "max_tokens": 5,
        "temperature": 0.1,
        "stream": True
    }
    
    print(f"\n=== Request {request_num} ===")
    
    try:
        response = requests.post(url, headers=headers, json=data, stream=True, timeout=30)
        
        if response.status_code != 200:
            print(f"ERROR: HTTP {response.status_code}")
            print(f"Response: {response.text}")
            return False
        
        print("SUCCESS: Request initiated")
        
        # Read the streaming response
        full_response = ""
        chunk_count = 0
        
        for line in response.iter_lines():
            if line:
                line_str = line.decode('utf-8')
                print(f"Chunk {chunk_count}: {line_str}")
                chunk_count += 1
                
                # Parse SSE format
                if line_str.startswith('data: '):
                    data_content = line_str[6:]  # Remove 'data: ' prefix
                    if data_content == '[DONE]':
                        print("Stream completed with [DONE]")
                        break
                    try:
                        chunk_data = json.loads(data_content)
                        if 'choices' in chunk_data and len(chunk_data['choices']) > 0:
                            if 'delta' in chunk_data['choices'][0]:
                                if 'content' in chunk_data['choices'][0]['delta']:
                                    content = chunk_data['choices'][0]['delta']['content']
                                    full_response += content
                    except json.JSONDecodeError:
                        print(f"Failed to parse JSON: {data_content}")
        
        print(f"Full response: '{full_response}'")
        print(f"Total chunks received: {chunk_count}")
        return True
        
    except requests.exceptions.RequestException as e:
        print(f"ERROR: Request failed - {e}")
        return False
    except Exception as e:
        print(f"ERROR: Unexpected error - {e}")
        return False

def test_health_endpoint():
    """Test the health endpoint to see if server is running."""
    try:
        response = requests.get("http://127.0.0.1:8080/health", timeout=5)
        if response.status_code == 200:
            print("✓ Server is running and healthy")
            return True
        else:
            print(f"✗ Health check failed with status {response.status_code}")
            return False
    except requests.exceptions.RequestException as e:
        print(f"✗ Cannot connect to server: {e}")
        return False

def main():
    print("Testing Model Server Streaming API")
    print("==================================")
    
    # First check if server is running
    if not test_health_endpoint():
        print("\nServer is not running. Please start the model server first:")
        print("./target/debug/model-server --model-path \"/Users/bya/Downloads/2f5623ae-0f29-404e-a80f-d2f357d8817a\" --architecture llama --port 8080 --model-id test-llama --app-data-dir \"/Volumes/zData/Projects/react-test/.ziee\"")
        sys.exit(1)
    
    # Test multiple consecutive streaming requests
    num_tests = 5
    successful_tests = 0
    
    for i in range(1, num_tests + 1):
        if test_streaming_request(i):
            successful_tests += 1
        else:
            print(f"Request {i} FAILED")
        
        # Wait a bit between requests
        if i < num_tests:
            time.sleep(2)
    
    print(f"\n=== SUMMARY ===")
    print(f"Successful requests: {successful_tests}/{num_tests}")
    
    if successful_tests == num_tests:
        print("✓ All streaming requests succeeded!")
        print("✓ No evidence of second request failures")
    elif successful_tests == 0:
        print("✗ All requests failed - server may not be responding")
    else:
        print(f"⚠ {num_tests - successful_tests} requests failed")
        print("This may indicate an issue with consecutive streaming requests")

if __name__ == "__main__":
    main()