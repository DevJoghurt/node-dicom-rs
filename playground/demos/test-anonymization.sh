#!/bin/bash
# Test script - run server and send files in sequence

echo "Starting anonymization server..."
node playground/demos/storescp-anonymization-demo.mjs &
SERVER_PID=$!

echo "Server PID: $SERVER_PID"
echo "Waiting 3 seconds for server to start..."
sleep 3

echo ""
echo "Sending files..."
node playground/demos/send-to-anon.mjs

echo ""
echo "Waiting 2 seconds for processing..."
sleep 2

echo ""
echo "Stopping server..."
kill $SERVER_PID

echo ""
echo "Checking logs..."
wait $SERVER_PID
