# Pydantic model fixtures
from pydantic import BaseModel
from typing import Optional, List

class User(BaseModel):
    id: str
    email: str
    name: Optional[str] = None
    age: int = 0

class Post(BaseModel):
    id: str
    title: str
    content: str
    author: User
    tags: List[str] = []
