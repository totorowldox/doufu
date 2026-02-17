export type Selection = string | string[] | null;

export type Handle = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

export type Region = {
  id: string;
  x: number; // 0-1 relative to viewer width
  y: number; // 0-1 relative to viewer height
  width: number; // 0-1
  height: number; // 0-1
  label: string;
  color: string;
  pageIndex: number;
  start?: number; // seconds
};

export type RegionFilter = "current" | "all";
