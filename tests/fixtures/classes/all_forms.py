class Animal:
    def __init__(self, name: str):
        self.name = name

    def speak(self) -> str:
        return f"{self.name} makes a sound"


class Dog(Animal):
    def __init__(self, name: str, breed: str):
        super().__init__(name)
        self.breed = breed

    def speak(self) -> str:
        return f"{self.name} barks"


class Utility:
    @staticmethod
    def add(a: int, b: int) -> int:
        return a + b

    @classmethod
    def create(cls) -> "Utility":
        return cls()
