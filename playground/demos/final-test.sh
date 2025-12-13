#!/bin/bash

echo "========================================"
echo "DICOM Anonymization Test"
echo "========================================"
echo

# Clean up
rm -rf playground/test-received-anon

# Kill any existing servers
lsof -ti:11115 | xargs -r kill -9 2>/dev/null

# Start server in background
echo "Starting anonymization server..."
node playground/demos/storescp-anonymization-demo.mjs > /tmp/anon-server.log 2>&1 &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait for server to start
sleep 3

# Send files
echo "Sending 5 DICOM files..."
node playground/demos/send-to-anon.mjs 2>&1 | grep -E "(File sent|Transfer Complete|Success)"

# Wait for processing
sleep 2

# Stop server
echo
echo "Stopping server..."
kill $SERVER_PID 2>/dev/null

# Verify results
echo
echo "========================================"
echo "Verification Results"
echo "========================================"
echo
node playground/demos/verify-anonymization.mjs

echo
echo "✅ Test complete!"
