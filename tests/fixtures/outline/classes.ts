export class Animal {
    name: string;

    constructor(name: string) {
        this.name = name;
    }

    speak(): void {
        console.log(`${this.name} makes a sound.`);
    }

    move(): void {
        console.log(`${this.name} moves.`);
    }
}
