def add(a: int, b: int) -> int:
    return a + b


class Calculator:
    def __init__(self):
        self.value = 0

    def add(self, n: int) -> None:
        self.value += n

    def get_value(self) -> int:
        return self.value
