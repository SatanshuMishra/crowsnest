<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    description?: string;
    children: Snippet;
  }

  let { title, description, children }: Props = $props();
  const id = $props.id();
</script>

<section
  class="section"
  aria-labelledby="{id}-title"
  aria-describedby={description ? `${id}-description` : undefined}
>
  <header>
    <h2 id="{id}-title">{title}</h2>
    {#if description}
      <p id="{id}-description">{description}</p>
    {/if}
  </header>
  <div class="body">
    {@render children()}
  </div>
</section>

<style>
  .section {
    display: grid;
    gap: var(--space-4);
    padding: var(--space-5) 0 var(--space-6);
    border-top: 1px solid var(--border);
  }

  header {
    display: grid;
    gap: var(--space-1);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--fs-lg);
    font-weight: var(--weight-strong);
  }

  p {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .body {
    display: grid;
    gap: var(--space-3);
  }
</style>
