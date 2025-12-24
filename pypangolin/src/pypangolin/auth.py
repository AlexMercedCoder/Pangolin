import requests
from .exceptions import AuthenticationError, PangolinError

def login(uri: str, username: str, password: str) -> str:
    """
    Exchange credentials for a JWT token using POST /api/v1/users/login
    """
    url = f"{uri.rstrip('/')}/api/v1/users/login"
    try:
        response = requests.post(url, json={"username": username, "password": password})
        
        if response.status_code == 200:
            data = response.json()
            return data["token"]
        elif response.status_code == 401:
            raise AuthenticationError("Invalid username or password", status_code=401)
        else:
            raise PangolinError(
                f"Login failed: {response.text}", 
                status_code=response.status_code,
                response_body=response.text
            )
            
    except requests.RequestException as e:
        raise PangolinError(f"Connection failed: {str(e)}")
