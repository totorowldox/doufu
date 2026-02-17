<script lang="ts">
  import type { Region, RegionFilter } from "$lib/types";

  export let scoreImagesCount = 0;
  export let regions: Region[] = [];
  export let currentRegions: Region[] = [];
  export let visibleRegions: Region[] = [];

  export let selectedRegionId: string | null = null;
  export let currentPlayingRegionId: string | null = null;

  export let regionFilter: RegionFilter = "current";
  export let formatSeconds: (value?: number) => string;

  export let onFilterChange: (value: RegionFilter) => void;
  export let onFocusRegion: (region: Region) => void;
  export let onUpdateRegion: (id: string, patch: Partial<Region>) => void;
  export let onFillWidth: (region: Region) => void;
  export let onExportRegion: (region: Region) => void;
  export let onRemoveRegion: (id: string) => void;
  export let onApplyCurrentTime: (id: string) => void;
</script>

<aside class="side-panel">
  <div class="section-heading">
    <div>
      <h2>裁剪区域与时间</h2>
      <span class="muted">切换页码时默认选择当前页区域</span>
    </div>
    <div class="filter">
      <label class="muted" for="region-filter">显示</label>
      <select
        id="region-filter"
        value={regionFilter}
        on:change={(e) =>
          onFilterChange(
            (e.currentTarget as HTMLSelectElement).value as RegionFilter,
          )}
      >
        <option value="current">当前页</option>
        <option value="all">全部</option>
      </select>
    </div>
  </div>
  {#if !scoreImagesCount}
    <p class="placeholder">先选择图片谱面后再添加裁剪区域</p>
  {:else if !regions.length}
    <p class="placeholder">暂无区域，点击“添加裁剪区域”开始</p>
  {:else if regionFilter === "current" && !currentRegions.length}
    <p class="placeholder">当前页暂无区域，可切换到“全部”查看</p>
  {:else}
    <div class="region-list">
      {#each visibleRegions as region}
        <div
          class={`region-item ${region.id === selectedRegionId ? "active" : ""} ${region.id === currentPlayingRegionId ? "playing" : ""}`}
          on:click={() => onFocusRegion(region)}
        >
          <div class="region-row">
            <input
              class="text"
              value={region.label}
              on:click|stopPropagation
              on:input={(e) =>
                onUpdateRegion(region.id, {
                  label: (e.currentTarget as HTMLInputElement).value,
                })}
            />
            <div class="color-dot" style={`background:${region.color}`}></div>
            <button
              type="button"
              class="ghost"
              on:click|stopPropagation={() => onFillWidth(region)}
            >
              横向撑满
            </button>
            <button
              type="button"
              class="ghost"
              on:click|stopPropagation={() => onExportRegion(region)}
            >
              导出截图
            </button>
            <button
              type="button"
              class="ghost"
              on:click|stopPropagation={() => onRemoveRegion(region.id)}
            >
              删除
            </button>
          </div>
          <div class="region-row">
            <label>开始</label>
            <input
              type="number"
              min="0"
              step="0.1"
              value={region.start ?? ""}
              on:click|stopPropagation
              on:input={(e) =>
                onUpdateRegion(region.id, {
                  start: Number((e.currentTarget as HTMLInputElement).value),
                })}
            />
            <button
              type="button"
              class="ghost"
              on:click|stopPropagation={() => onApplyCurrentTime(region.id)}
            >
              取当前时间
            </button>
            <span class="muted">{formatSeconds(region.start)}</span>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</aside>
