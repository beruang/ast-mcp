# Python decorator fixtures
import pytest

@app.get("/users")
async def list_users():
    pass

@pytest.fixture
def db_session():
    pass

from dataclasses import dataclass

@dataclass
class User:
    id: str
    name: str

def my_decorator(func):
    return func

@my_decorator
def decorated_function():
    pass
