
import requests
import subprocess
import time
import os
import signal
import sys
import threading

# Configuration
MOCK_PORT = 34567
BACKEND_PORT = 8080
BASE_URL = f"http://localhost:{BACKEND_PORT}"
MOCK_URL = f"http://localhost:{MOCK_PORT}"

# Environment for Backend
ENV = os.environ.copy()
ENV.update({
    "PANGOLIN_ROOT_USER": "admin",
    "PANGOLIN_ROOT_PASSWORD": "admin123",
    "PANGOLIN_JWT_SECRET": "secret123",
    "OAUTH_GOOGLE_CLIENT_ID": "mock_client",
    "OAUTH_GOOGLE_CLIENT_SECRET": "mock_secret",
    "OAUTH_GOOGLE_REDIRECT_URI": f"{BASE_URL}/oauth/callback/google",
    "OAUTH_GOOGLE_AUTH_URL": f"{MOCK_URL}/authorize",
    "OAUTH_GOOGLE_TOKEN_URL": f"{MOCK_URL}/token",
    "OAUTH_GOOGLE_USERINFO_URL": f"{MOCK_URL}/userinfo",
})

def stream_logs(process, name):
    for line in iter(process.stdout.readline, b''):
        print(f"[{name}] {line.decode().strip()}")

def main():
    print("Starting OAuth Authorization Flow Test...")
    
    # 1. Start Mock Provider
    print(f"Starting Mock OAuth Provider on port {MOCK_PORT}...")
    mock_proc = subprocess.Popen(
        ["python3", "mock_oauth_provider.py"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    # Give it a second
    time.sleep(1)
    if mock_proc.poll() is not None:
        print("Mock provider failed to start")
        print("STDERR:", mock_proc.stderr.read().decode())
        exit(1)
        
    # 2. Start Backend
    print(f"Starting Pangolin Backend on port {BACKEND_PORT}...")
    backend_proc = subprocess.Popen(
        ["./target/debug/pangolin_api"],
        cwd="pangolin",
        env=ENV,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    
    # Stream logs in background threads
    threading.Thread(target=stream_logs, args=(mock_proc, "MOCK"), daemon=True).start()
    threading.Thread(target=stream_logs, args=(backend_proc, "BACKEND"), daemon=True).start()
    
    print("Waiting for backend to start...", flush=True)
    max_retries = 90 # Increase to 90s just in case
    for i in range(max_retries):
        try:
            requests.get(f"{BASE_URL}/api/v1/app-config")
            print("Backend is ready!", flush=True)
            break
        except requests.exceptions.ConnectionError:
            if i % 5 == 0:
                print(f"Waiting for backend... ({i}/{max_retries})", flush=True)
            if i == max_retries - 1:
                print("Backend failed to start in time.", flush=True)
                exit(1)
            time.sleep(1) # Wait 1s before retry

    
    try:
        # 3. Simulate Client
        
        # A. Initiate Auth
        # GET /oauth/authorize/google?redirect_uri=... (Client supplied return URL, e.g. frontend)
        # Note: The 'redirect_uri' query param here is where Pangolin should redirect AFTER successful login.
        # It is NOT the OAuth callback URI.
        client_redirect = "http://localhost:3000/dashboard"
        init_url = f"{BASE_URL}/oauth/authorize/google?redirect_uri={client_redirect}"
        
        print(f"Client requesting: {init_url}")
        # Dont allow redirects automatically so we can inspect intermediate steps
        r1 = requests.get(init_url, allow_redirects=False)
        
        if r1.status_code != 302 and r1.status_code != 303:
            print(f"FAIL: Expected redirect from init, got {r1.status_code}")
            print(r1.text)
            exit(1)
            
        location_auth = r1.headers["Location"]
        print(f"Redirected to Provider: {location_auth}")
        
        if MOCK_URL not in location_auth:
            print(f"FAIL: Redirect location does not point to mock provider: {location_auth}")
            exit(1)
            
        # B. User Authenticates at Provider (Mock just redirects back)
        r2 = requests.get(location_auth, allow_redirects=False)
        
        if r2.status_code != 302 and r2.status_code != 303:
             print(f"FAIL: Expected redirect from provider, got {r2.status_code}")
             exit(1)
             
        location_callback = r2.headers["Location"]
        print(f"Provider redirected back to Callback: {location_callback}")
        
        if "/oauth/callback/google" not in location_callback:
            print("FAIL: Provider did not redirect to correct callback URL")
            exit(1)
            
        # C. Client follows redirect to Backend Callback
        # This triggers the code exchange and user info fetch on backend
        r3 = requests.get(location_callback, allow_redirects=False)
        
        # The backend should process OAuth and then redirect to the `client_redirect` with token?
        # Or return JSON?
        # Let's check `oauth_handlers.rs` behavior.
        # Usually it redirects to `redirect_uri` (from state) with `token` query param.
        
        print(f"Callback response status: {r3.status_code}")
        if r3.status_code == 302 or r3.status_code == 303:
             print(f"Final Redirect Location: {r3.headers['Location']}")
             if client_redirect in r3.headers['Location']:
                 print("PASS: Redirected to client application!")
                 if "token=" in r3.headers['Location']:
                     print("PASS: Token present in redirect URL")
                 else:
                     print("FAIL: Token missing from redirect URL")
                     exit(1)
             else:
                 print(f"FAIL: Redirected to wrong location: {r3.headers['Location']}")
                 exit(1)
        else:
             print(f"FAIL: Callback did not redirect. Body: {r3.text}")
             exit(1)
             
    except Exception as e:
        print(f"Error: {e}")
        exit(1)
    finally:
        print("Cleaning up processes...")
        mock_proc.terminate()
        backend_proc.terminate()
        mock_proc.wait()
        backend_proc.wait()
        
    print("OAuth Flow Test Complete")

if __name__ == "__main__":
    main()
