<script lang="ts">
  import { onMount } from "svelte";
  import { windowFrame } from "../lib/window";

  let maximized = $state(false);

  onMount(() => {
    let active = true;
    let stop: (() => void) | undefined;
    const sync = () => {
      windowFrame
        .isMaximized()
        .then((value) => {
          maximized = value;
        })
        .catch(() => undefined);
    };
    sync();
    windowFrame
      .onResized(sync)
      .then((unlisten) => {
        if (active) {
          stop = unlisten;
        } else {
          unlisten();
        }
      })
      .catch(() => undefined);
    return () => {
      active = false;
      stop?.();
    };
  });

  const sizeLabel = $derived(maximized ? "Restore" : "Maximize");
</script>

<div class="controls">
  <button type="button" class="control" aria-label="Minimize" title="Minimize" onclick={windowFrame.minimize}>
    <svg viewBox="0 0 10 10" aria-hidden="true" focusable="false"><path d="M0 5.5h10" /></svg>
  </button>
  <button type="button" class="control" aria-label={sizeLabel} title={sizeLabel} onclick={windowFrame.toggleMaximize}>
    <svg viewBox="0 0 10 10" aria-hidden="true" focusable="false">
      {#if maximized}
        <path d="M2.5 2.5V.5h7v7h-2M.5 2.5h7v7h-7Z" />
      {:else}
        <path d="M.5.5h9v9h-9Z" />
      {/if}
    </svg>
  </button>
  <button type="button" class="control close" aria-label="Close" title="Close" onclick={windowFrame.close}>
    <svg viewBox="0 0 10 10" aria-hidden="true" focusable="false"><path class="cross" d="M.5.5l9 9m0-9l-9 9" /></svg>
  </button>
</div>

<style>
  .controls {
    display: flex;
    flex: none;
    align-self: stretch;
  }

  .control {
    display: grid;
    place-items: center;
    width: var(--window-control-width);
    padding: 0;
    border: 0;
    border-radius: 0;
    background: transparent;
    color: var(--text-muted);
    cursor: default;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }

  .control:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .control:active {
    background: color-mix(in srgb, var(--text) 8%, transparent);
  }

  .close:hover,
  .close:active {
    background: var(--window-close);
    color: var(--window-close-text);
  }

  .close:active {
    background: color-mix(in srgb, var(--window-close) 88%, transparent);
  }

  .control:focus-visible {
    outline: 2px solid var(--focus);
    outline-offset: -2px;
  }

  svg {
    width: 10px;
    height: 10px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1;
    shape-rendering: crispEdges;
  }

  .cross {
    shape-rendering: geometricPrecision;
  }
</style>
