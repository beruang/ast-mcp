# 1. Simple function
def greet(name: str) -> str:
    return f"Hello, {name}"

# 2. Function with default parameter
def repeat(text: str, times: int = 1) -> str:
    return text * times

# 3. Async function
async def fetch_data(url: str) -> dict:
    return {}

# 4. Method in class
class Calculator:
    def add(self, a: int, b: int) -> int:
        return a + b

    def get_value(self) -> int:
        return 0
