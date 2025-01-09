#!/usr/bin/env bash

# Exit on error
set -e

# Directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Install Python dependencies
pip install -r requirements.txt

# Function to cleanup processes on exit
cleanup() {
    echo "Cleaning up..."
    # Kill Python mock servers
    pkill -f "python.*mock_servers.py" || true
    # Stop degressly-core container
    docker stop degressly-core || true
    docker rm degressly-core || true
}
trap cleanup EXIT

# Start mock servers
echo "Starting mock servers..."
python mock_servers.py &
sleep 5  # Wait for servers to start

# Test with different wait times
test_wait_time() {
    local wait_time=$1
    echo "Testing with wait_after_forwarding_to_primary=$wait_time"
    
    # Stop previous container if exists
    docker stop degressly-core || true
    docker rm degressly-core || true
    
    # Start degressly-core with configured wait time
    echo "Starting degressly-core..."
    docker run -d --name degressly-core \
        -p 8000:8000 \
        -e wait_after_forwarding_to_primary="$wait_time" \
        -e primary_host="http://host.docker.internal:9000" \
        -e secondary_host="http://host.docker.internal:9001" \
        -e candidate_host="http://host.docker.internal:9002" \
        --add-host=host.docker.internal:host-gateway \
        degressly-core:latest
    
    # Wait for service to start
    sleep 5
    
    # Send test request
    echo "Sending test request..."
    curl -i -X POST http://localhost:8000/test-endpoint \
        -H "Content-Type: application/json" \
        -d '{"test":"payload"}'
    
    # Get logs to check timing
    echo "Container logs:"
    docker logs degressly-core
    
    # Wait to collect all logs
    sleep 2
}

# Run tests with different wait times
for wait_time in 0 100 -100; do
    echo "=== Testing with wait time: $wait_time ==="
    test_wait_time $wait_time
    echo "=== Completed test with wait time: $wait_time ==="
    echo
done

echo "All tests completed"
