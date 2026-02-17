<script lang="ts">
  export let scoreImages: string[] = [];
  export let activeIndex = 0;
  export let fileName: (path: string) => string;
  export let onSelect: (index: number) => void;
</script>

{#if scoreImages.length}
  <div class="file-chip">已选图片 {scoreImages.length} 张</div>
  <div class="thumbs">
    {#each scoreImages as img, idx}
      <button
        type="button"
        class:selected={idx === activeIndex}
        on:click={() => onSelect(idx)}
        title={fileName(img)}
      >
        {idx + 1}
      </button>
    {/each}
  </div>
{/if}
