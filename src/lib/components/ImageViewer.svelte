<script lang="ts">
  import type { Handle, Region } from "$lib/types";
  import { clamp, MIN_SIZE } from "$lib/utils";
  import { onDestroy } from "svelte";

  export let imagePath: string;
  export let regions: Region[] = [];
  export let selectedRegionId: string | null = null;
  export let currentPlayingRegionId: string | null = null;
  export let onSelectRegion: (id: string) => void;

  export let toUrl: (path: string) => string;
  export let updateRegion: (id: string, patch: Partial<Region>) => void;
  export let onImageLoad: (event: Event) => void;

  let overlayRef: HTMLDivElement | null = null;

  type DragState = {
    id: string;
    type: "move" | "resize";
    handle?: Handle;
    startX: number;
    startY: number;
    start: Region;
    rect: DOMRect;
  } | null;

  let dragging: DragState = null;

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    e.preventDefault();
    const { id, type, handle, startX, startY, start, rect } = dragging;
    const dx = (e.clientX - startX) / rect.width;
    const dy = (e.clientY - startY) / rect.height;

    let { x, y, width, height } = start;

    if (type === "move") {
      x = clamp(start.x + dx, 0, 1 - start.width);
      y = clamp(start.y + dy, 0, 1 - start.height);
    } else if (handle) {
      if (handle.includes("e")) {
        width = clamp(start.width + dx, MIN_SIZE, 1 - start.x);
      }
      if (handle.includes("s")) {
        height = clamp(start.height + dy, MIN_SIZE, 1 - start.y);
      }
      if (handle.includes("w")) {
        const newX = clamp(start.x + dx, 0, start.x + start.width - MIN_SIZE);
        width = start.width + (start.x - newX);
        x = newX;
      }
      if (handle.includes("n")) {
        const newY = clamp(start.y + dy, 0, start.y + start.height - MIN_SIZE);
        height = start.height + (start.y - newY);
        y = newY;
      }
      width = clamp(width, MIN_SIZE, 1 - x);
      height = clamp(height, MIN_SIZE, 1 - y);
    }

    updateRegion(id, { x, y, width, height });
  }

  function stopDrag() {
    dragging = null;
    window.removeEventListener("pointermove", onPointerMove);
  }

  function startDrag(
    e: PointerEvent,
    region: Region,
    type: "move" | "resize",
    handle?: Handle,
  ) {
    if (!overlayRef) return;
    e.preventDefault();
    e.stopPropagation();
    const rect = overlayRef.getBoundingClientRect();
    dragging = {
      id: region.id,
      type,
      handle,
      startX: e.clientX,
      startY: e.clientY,
      start: { ...region },
      rect,
    };
    window.addEventListener("pointermove", onPointerMove, { passive: false });
    window.addEventListener("pointerup", stopDrag, { once: true });
  }

  function handleSelect(regionId: string) {
    onSelectRegion(regionId);
  }

  onDestroy(() => {
    window.removeEventListener("pointermove", onPointerMove);
  });
</script>

{#if imagePath}
  <div class="viewer image">
    <div class="image-stage">
      <div class="image-scroll">
        <div class="image-stack">
          <img src={toUrl(imagePath)} alt="score" on:load={onImageLoad} />
          <div class="overlay" bind:this={overlayRef}>
            {#each regions as region}
              <div
                role="region"
                class={`region ${region.id === selectedRegionId ? "selected" : ""} ${region.id === currentPlayingRegionId ? "playing" : ""}`}
                style={`left:${region.x * 100}%;top:${region.y * 100}%;width:${region.width * 100}%;height:${region.height * 100}%;border-color:${region.color};`}
                on:pointerdown={(e) => {
                  handleSelect(region.id);
                  startDrag(e, region, "move");
                }}
              >
                <div class="label" style={`background:${region.color}`}>
                  {region.label}
                </div>
                <div class="handles">
                  {#each ["nw", "n", "ne", "e", "se", "s", "sw", "w"] as h}
                    <div
                      role="button"
                      class={`handle handle-${h}`}
                      on:pointerdown={(e) => {
                        handleSelect(region.id);
                        startDrag(e, region, "resize", h as Handle);
                      }}
                    ></div>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
