<script lang="ts">
  export let tauriReady = false;
  export let scorePdf: string | null = null;
  export let toUrl: (path: string) => string;
  export let fileName: (path: string) => string;
</script>

{#if scorePdf}
  <div class="file-chip">PDF：{fileName(scorePdf)}</div>
  <div class="viewer pdf">
    {#if tauriReady}
      <iframe title="score-pdf" src={toUrl(scorePdf)} allowfullscreen></iframe>
    {:else}
      <p class="placeholder">正在连接本地资源…</p>
    {/if}
  </div>
{/if}
