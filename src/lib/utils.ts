import type { Region } from "$lib/types";

export const imageFilter = {
  name: "Images",
  extensions: ["png", "jpg", "jpeg", "webp"],
} as const;

export const pdfFilter = { name: "PDF", extensions: ["pdf"] } as const;

export const audioFilter = {
  name: "Audio",
  extensions: ["mp3", "wav", "flac", "ogg", "m4a", "aac"],
} as const;

export const REGION_COLOR = "#22d3ee";
export const MIN_SIZE = 0.04;

export function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}

export function clampRegion(region: Region) {
  const x = clamp(region.x, 0, 1 - MIN_SIZE);
  const y = clamp(region.y, 0, 1 - MIN_SIZE);
  const width = clamp(region.width, MIN_SIZE, 1 - x);
  const height = clamp(region.height, MIN_SIZE, 1 - y);
  return { ...region, x, y, width, height };
}

export function fileName(path: string) {
  return path.split(/[\\/]/).pop() ?? path;
}

export function formatSeconds(value?: number) {
  if (value == null || Number.isNaN(value)) return "-";
  const v = Math.max(0, value);
  const m = Math.floor(v / 60);
  const s = (v % 60).toFixed(2).padStart(5, "0");
  return `${m}:${s}`;
}
