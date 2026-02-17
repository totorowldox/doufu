<script lang="ts">
  export let title: string;
  export let hint: string;

  export let onPickScore: () => void;
  export let onPickAudio: () => void;
</script>

<header class="section header">
  <div>
    <h1>{title}</h1>
    <p class="hint">{hint}</p>
  </div>
  <div class="actions">
    <button type="button" on:click={onPickScore}>选择谱面 (PNG/PDF)</button>
    <button type="button" on:click={onPickAudio}>选择音频</button>
  </div>
</header>
