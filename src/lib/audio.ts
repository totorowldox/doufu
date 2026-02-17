import type { Region } from "$lib/types";

type TimerHandle = ReturnType<typeof setInterval> | null;

let timer: TimerHandle = null;

export function startAudioTimer(onTick: () => void, intervalMs = 16) {
  stopAudioTimer();
  timer = setInterval(onTick, intervalMs);
}

export function stopAudioTimer() {
  if (timer) {
    clearInterval(timer);
    timer = null;
  }
}

export function getCurrentPlayingRegion(
  regions: Region[],
  audioTime: number,
): Region | null {
  if (!regions.length) return null;

  let left = 0;
  let right = regions.length - 1;

  while (left <= right) {
    const mid = Math.floor((left + right) / 2);
    const midStart = regions[mid].start ?? 0;
    const nextStart =
      mid + 1 < regions.length
        ? regions[mid + 1].start ?? Number.POSITIVE_INFINITY
        : Number.POSITIVE_INFINITY;

    if (audioTime >= midStart && audioTime < nextStart) {
      return regions[mid];
    } else if (audioTime < midStart) {
      right = mid - 1;
    } else {
      left = mid + 1;
    }
  }

  return regions[regions.length - 1] ?? null;
}
