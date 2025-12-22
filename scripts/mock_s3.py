from http.server import HTTPServer, BaseHTTPRequestHandler
import sys

storage = {}

class MockS3(BaseHTTPRequestHandler):
    def do_PUT(self):
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length)
        storage[self.path] = body
        self.send_response(200)
        self.send_header('ETag', '"fake-etag"')
        self.end_headers()
        
    def do_GET(self):
        if self.path in storage:
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(storage[self.path])
        else:
            self.send_response(404)
            self.end_headers()
            
    def do_HEAD(self):
        if self.path in storage:
            self.send_response(200)
            self.send_header('Content-Length', str(len(storage[self.path])))
            self.end_headers()
        else:
            self.send_response(404)
            self.end_headers()
            
    def do_POST(self):
        # Multipart uploads or other POSTs
        self.send_response(200)
        self.end_headers()

server = HTTPServer(('127.0.0.1', 9005), MockS3)
print("Mock S3 listening on 9005")
server.serve_forever()
