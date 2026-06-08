class Animal:
    def __init__(self, name: str):
        self.name = name

    def speak(self) -> None:
        print(f"{self.name} makes a sound.")

    def move(self) -> None:
        print(f"{self.name} moves.")
