// 1. Function declaration
function greet(name: string): string {
    return `Hello, ${name}`;
}

// 2. Generator function
function* range(start: number, end: number): Generator<number> {
    for (let i = start; i < end; i++) {
        yield i;
    }
}

// 3. Arrow function
const double = (x: number): number => x * 2;

// 4. Async function
async function fetchData(url: string): Promise<string> {
    const response = await fetch(url);
    return response.text();
}

// 5. Exported function
export function add(a: number, b: number): number {
    return a + b;
}

// 6. Function expression
const square = function(x: number): number {
    return x * x;
};

// 7. Method in class
class MathUtil {
    multiply(a: number, b: number): number {
        return a * b;
    }

    static identity<T>(x: T): T {
        return x;
    }
}
