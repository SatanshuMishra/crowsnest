<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title?: string;
    children: Snippet;
  }

  let { title, children }: Props = $props();
  const id = $props.id();
</script>

<div class="group" role="group" aria-labelledby={title ? `${id}-title` : undefined}>
  {#if title}
    <h3 id="{id}-title">{title}</h3>
  {/if}
  <div class="body">
    {@render children()}
  </div>
</div>

<style>
  .group {
    display: grid;
    gap: var(--space-2);
  }

  .group + :global(.group) {
    padding-top: var(--space-4);
    border-top: 1px solid var(--border);
  }

  h3 {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
    font-weight: var(--weight-medium);
  }

  .body {
    display: grid;
    gap: var(--space-1);
  }
</style>
