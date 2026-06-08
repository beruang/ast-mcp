__all__ = ["greet", "AsyncWorker"]

import os

def greet(name: str) -> str:
    return f"Hello, {name}!"

class AsyncWorker:
    async def run(self):
        pass

async def fetch_data():
    pass
