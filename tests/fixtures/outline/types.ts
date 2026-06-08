export interface Printable {
    print(): void;
}

export type Point = {
    x: number;
    y: number;
};

export enum Color {
    Red,
    Green,
    Blue,
}
