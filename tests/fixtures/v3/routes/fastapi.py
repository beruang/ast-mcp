# FastAPI route fixtures
from fastapi import APIRouter

router = APIRouter()

@app.get("/users")
async def list_users():
    return []

@app.post("/users")
async def create_user():
    return {}

@app.get("/users/{id}")
async def get_user(id: str):
    return {"id": id}

@router.get("/items")
async def list_items():
    return []

@router.post("/items")
async def create_item():
    return {}
