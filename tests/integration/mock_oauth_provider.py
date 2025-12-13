
import http.server
import socketserver
import urllib.parse
import json
import time

PORT = 9090

class MockOAuthHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path
        query = urllib.parse.parse_qs(parsed.query)
        
        if path == "/authorize":
            # Simulate Authorization Endpoint
            # Redirect back to redirect_uri with code and state
            redirect_uri = query.get("redirect_uri", [""])[0]
            state = query.get("state", [""])[0]
            
            if not redirect_uri:
                self.send_error(400, "Missing redirect_uri")
                return
                
            code = "mock_auth_code_12345"
            
            # Construct redirect URL
            target_url = f"{redirect_uri}?code={code}&state={state}"
            
            print(f"Authorize called. Redirecting to: {target_url}")
            
            self.send_response(302)
            self.send_header("Location", target_url)
            self.end_headers()
            return
            
        elif path == "/userinfo":
            # Simulate UserInfo Endpoint
            # Expect Authorization header
            auth_header = self.headers.get("Authorization")
            if not auth_header or not auth_header.startswith("Bearer "):
                self.send_error(401, "Missing or invalid Authorization header")
                return
                
            token = auth_header.split(" ")[1]
            if token != "mock_access_token_12345":
                self.send_error(403, "Invalid token")
                return
                
            user_profile = {
                "sub": "mock_user_123",
                "name": "Mock User",
                "email": "mockuser@example.com",
                "email_verified": True
            }
            
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(user_profile).encode())
            return
            
        else:
            self.send_error(404, "Not Found")
            
    def do_POST(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path
        
        if path == "/token":
            # Simulate Token Endpoint
            length = int(self.headers.get('content-length', 0))
            body = self.rfile.read(length).decode()
            params = urllib.parse.parse_qs(body)
            
            code = params.get("code", [""])[0]
            if code != "mock_auth_code_12345":
                self.send_error(400, "Invalid code")
                return
                
            token_response = {
                "access_token": "mock_access_token_12345",
                "token_type": "Bearer",
                "expires_in": 3600,
                "id_token": "mock_id_token_jwt" # In real world this is a JWT
            }
            
            print("Token called. Returning mock token.")
            
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(token_response).encode())
            return
            
        else:
            self.send_error(404, "Not Found")

PORT = 34567

class ReusableTCPServer(socketserver.TCPServer):
    allow_reuse_address = True

print(f"Starting Mock OAuth Provider on port {PORT}...")
with ReusableTCPServer(("127.0.0.1", PORT), MockOAuthHandler) as httpd:
    httpd.serve_forever()
