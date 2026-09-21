import { describe, expect, it } from "vitest";
import { nextInDirection } from "./nav";

const r = (x: number, y: number, w = 100, h = 100) => ({ left: x, top: y, right: x + w, bottom: y + h, width: w, height: h, x, y, toJSON: () => ({}) }) as DOMRect;

describe("spatial navigation", () => {
  const grid = [r(0, 0), r(120, 0), r(240, 0), r(0, 120), r(120, 120), r(240, 120)];
  it("moves right/left along a row", () => {
    expect(nextInDirection(grid[0], grid, "right")).toBe(1);
    expect(nextInDirection(grid[1], grid, "right")).toBe(2);
    expect(nextInDirection(grid[2], grid, "left")).toBe(1);
  });
  it("moves down/up between rows keeping the column", () => {
    expect(nextInDirection(grid[1], grid, "down")).toBe(4);
    expect(nextInDirection(grid[5], grid, "up")).toBe(2);
  });
  it("returns -1 at an edge", () => {
    expect(nextInDirection(grid[2], grid, "right")).toBe(-1);
    expect(nextInDirection(grid[0], grid, "up")).toBe(-1);
  });
  it("prefers the aligned tile over a nearer diagonal one", () => {
    const from = r(0, 0);
    const aligned = r(300, 0); // primary 300, secondary 0 → score 300
    const diagonal = r(150, 200); // primary 150, secondary 200 → score 150 + 480
    expect(nextInDirection(from, [diagonal, aligned], "right")).toBe(1);
  });
});
