<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  interface Props extends HTMLButtonAttributes {
    variant?: "primary" | "secondary" | "ghost" | "danger";
    size?: "sm" | "md";
    busy?: boolean;
    children: Snippet;
  }

  let {
    variant = "secondary",
    size = "md",
    busy = false,
    disabled = false,
    type = "button",
    children,
    ...rest
  }: Props = $props();
</script>

<button
  {...rest}
  {type}
  class="button {variant} {size}"
  disabled={disabled || busy}
  aria-busy={busy || undefined}
>
  {@render children()}
</button>

<style>
  .button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    min-height: var(--target-min);
    padding: 0 var(--space-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text);
    font-size: var(--fs-md);
    font-weight: var(--weight-medium);
    white-space: nowrap;
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      translate var(--duration-fast) var(--ease-out);
  }

  .button:active:not(:disabled) {
    translate: 0 1px;
  }

  .button.md {
    min-height: 32px;
  }

  .button.sm {
    padding: 0 var(--space-2);
    font-size: var(--fs-sm);
  }

  .button:hover:not(:disabled) {
    background: var(--surface-hover);
  }

  .primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--accent-contrast);
  }

  .primary:hover:not(:disabled) {
    border-color: var(--accent-strong);
    background: var(--accent-strong);
  }

  .ghost {
    border-color: transparent;
  }

  .danger {
    color: var(--danger);
  }

  .button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .button[aria-busy="true"] {
    cursor: progress;
  }
</style>
