<script lang="ts">
  export let message: string;
</script>

{#if message}
  <div class="message">{message}</div>
{/if}
