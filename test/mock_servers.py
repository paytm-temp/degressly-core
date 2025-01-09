from flask import Flask, request, jsonify
import time
import os
import logging

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s.%(msecs)03d - %(name)s - %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)

def create_mock_server(name, port):
    app = Flask(name)
    logger = logging.getLogger(name)
    
    @app.route('/', defaults={'path': ''}, methods=['GET', 'POST', 'PUT', 'DELETE', 'PATCH'])
    @app.route('/<path:path>', methods=['GET', 'POST', 'PUT', 'DELETE', 'PATCH'])
    def catch_all(path):
        timestamp = time.time()
        request_data = {
            'method': request.method,
            'path': path,
            'headers': dict(request.headers),
            'timestamp': timestamp,
            'body': request.get_data(as_text=True)
        }
        
        logger.info(f"Received request: {request.method} /{path}")
        logger.info(f"Timestamp: {timestamp}")
        logger.info(f"Headers: {dict(request.headers)}")
        
        return jsonify({
            'status': 'success',
            'server': name,
            'timestamp': timestamp,
            'request': request_data
        })
    
    return app

def main():
    # Create mock servers
    primary = create_mock_server('primary', 9000)
    secondary = create_mock_server('secondary', 9001)
    candidate = create_mock_server('candidate', 9002)
    
    # Run servers
    from threading import Thread
    Thread(target=lambda: primary.run(host='0.0.0.0', port=9000)).start()
    Thread(target=lambda: secondary.run(host='0.0.0.0', port=9001)).start()
    Thread(target=lambda: candidate.run(host='0.0.0.0', port=9002)).start()

if __name__ == '__main__':
    main()
