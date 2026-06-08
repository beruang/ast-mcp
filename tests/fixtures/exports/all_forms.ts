// 1. Export function
export function add(a: number, b: number): number {
    return a + b;
}

// 2. Export class
export class Calculator {
    value: number = 0;
}

// 3. Export const
export const PI = 3.14159;

// 4. Export let
export let counter = 0;

// 5. Export var
export var version = "1.0";

// 6. Export type
export type Point = { x: number; y: number };

// 7. Export interface
export interface Shape {
    area(): number;
}

// 8. Export enum
export enum Color { Red, Green, Blue }

// 9. Default export function
export default function greet() { return "hello"; }

// 10. Default export expression
export default 42;

// 11. Re-export named
export { add as sum, Calculator };

// 12. Re-export from module
export { greet } from "./greetings";

// 13. Re-export all
export * from "./utils";
