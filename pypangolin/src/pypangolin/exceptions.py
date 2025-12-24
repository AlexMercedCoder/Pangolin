class PangolinError(Exception):
    """Base exception for Pangolin client"""
    def __init__(self, message: str, status_code: int = None, response_body: str = None):
        super().__init__(message)
        self.status_code = status_code
        self.response_body = response_body

class AuthenticationError(PangolinError):
    pass

class AuthorizationError(PangolinError):
    pass

class ForbiddenError(PangolinError):
    pass

class NotFoundError(PangolinError):
    pass

class ConflictError(PangolinError):
    """Resource already exists"""
    pass

class ValidationError(PangolinError):
    pass
