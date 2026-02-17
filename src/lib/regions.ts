import type { Region } from "./types";

export function createRegion(pageIndex: number, seq: number, color: string): Region {
  return {
    id:
      crypto.randomUUID?.() ??
      `${Date.now()}-${Math.random().toString(16).slice(2)}`,
    x: 0,
    y: 0.25,
    width: 1,
    height: 0.2,
    label: `${pageIndex + 1}-${seq}`,
    color,
    pageIndex,
  };
}
