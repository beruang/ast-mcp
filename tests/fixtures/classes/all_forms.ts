// 1. Simple class
class Animal {
    name: string;
    constructor(name: string) {
        this.name = name;
    }
    speak(): string {
        return `${this.name} makes a sound`;
    }
}

// 2. Class with extends
class Dog extends Animal {
    breed: string;
    constructor(name: string, breed: string) {
        super(name);
        this.breed = breed;
    }
    speak(): string {
        return `${this.name} barks`;
    }
}

// 3. Class with implements
interface Printable {
    print(): void;
}

class Document implements Printable {
    print(): void {
        console.log("printing...");
    }
}

// 4. Abstract class
abstract class Shape {
    abstract area(): number;
}

// 5. Exported class
export class ExportedUtil {
    static version = "1.0";
}

// 6. Default export class
export default class DefaultClass {
    value = 42;
}
