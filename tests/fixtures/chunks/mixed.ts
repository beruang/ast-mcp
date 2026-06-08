import { something } from "./module";

export const VERSION = "1.0.0";

function helper(x: number): number {
    return x * 2;
}

export function main(): void {
    const result = helper(42);
    console.log(result);
}

class Counter {
    private count: number = 0;

    increment(): void {
        this.count++;
    }

    getCount(): number {
        return this.count;
    }
}

export default Counter;
