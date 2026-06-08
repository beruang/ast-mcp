__all__ = ["add", "Calculator"]

def add(a: int, b: int) -> int:
    return a + b


class Calculator:
    def __init__(self):
        self.value = 0

    def add(self, n: int) -> int:
        self.value += n
        return self.value


def _private_helper():
    pass


class _PrivateClass:
    pass
