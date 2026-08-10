import requests

from .exceptions import AuthenticationError, PangolinError

#: B39: login had no timeout either, so a hung server blocked the very first
#: call a caller makes.
LOGIN_TIMEOUT = (5, 30)

def login(uri: str, username: str, password: str, tenant_id: str = None) -> str:
    """
    Exchange credentials for a JWT token using POST /api/v1/users/login
    """
    url = f"{uri.rstrip('/')}/api/v1/users/login"
    payload = {"username": username, "password": password}
    if tenant_id:
        payload["tenant-id"] = tenant_id

    try:
        response = requests.post(url, json=payload, timeout=LOGIN_TIMEOUT)
        
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
        raise PangolinError(f"Connection failed: {str(e)}") from e
