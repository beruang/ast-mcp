export class Calculator {
    value: number = 0;

    add(n: number): void {
        if (n > 0) {
            this.value += n;
        }
    }

    getValue(): number {
        return this.value;
    }
}
