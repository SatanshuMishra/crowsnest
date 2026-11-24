<script lang="ts">
  import { app } from "../lib/state.svelte";

  const pinned = $derived(app.settings?.alwaysOnTop ?? false);
  const hint = $derived(
    pinned ? "Crowsnest stays above other windows. Click to stop." : "Keep Crowsnest above other windows",
  );
</script>

{#if app.settings}
  <div class="pin">
    <button
      type="button"
      class="toggle"
      class:pinned
      aria-label="Keep on top"
      aria-pressed={pinned}
      title={hint}
      onclick={() => app.update({ alwaysOnTop: !pinned })}
    >
      <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true" focusable="false">
        <path class="needle" d="M5.25 2.25h5.5M8 10v4" />
        <path class="head" d="M6.5 2.25v3.9L4.5 10h7l-2-3.85v-3.9Z" />
      </svg>
    </button>
  </div>
{/if}

<style>
  .pin {
    display: flex;
    flex: none;
    align-items: center;
    align-self: stretch;
    padding: 0 var(--space-2);
    border-left: 1px solid var(--border);
  }

  .toggle {
    display: inline-grid;
    place-items: center;
    width: 32px;
    height: 32px;
    padding: 0;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }

  .toggle:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .pinned {
    border-color: var(--border-strong);
    background: var(--surface-hover);
    color: var(--text);
  }

  svg {
    flex: none;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.25;
    stroke-linejoin: round;
    stroke-linecap: round;
    rotate: 45deg;
    transition: rotate var(--duration-base) var(--ease-out);
  }

  .pinned svg {
    rotate: 0deg;
  }

  .pinned .head {
    fill: currentColor;
  }

  @media (prefers-reduced-motion: reduce) {
    svg {
      transition: none;
    }
  }
</style>
