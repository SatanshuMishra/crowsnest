<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";
  import Icon, { type IconName } from "./Icon.svelte";

  interface Props extends HTMLButtonAttributes {
    icon: IconName;
    label: string;
    pressed?: boolean;
    busy?: boolean;
  }

  let { icon, label, pressed, busy = false, disabled = false, type = "button", ...rest }: Props = $props();
</script>

<button
  {...rest}
  {type}
  class="icon-button"
  class:busy
  aria-label={label}
  title={label}
  aria-pressed={pressed}
  aria-busy={busy || undefined}
  {disabled}
>
  <Icon name={icon} size={18} />
</button>

<style>
  .icon-button {
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
      color var(--duration-fast) var(--ease-out),
      scale var(--duration-fast) var(--ease-out);
  }

  .icon-button:hover:not(:disabled, [aria-disabled="true"]),
  .icon-button[aria-pressed="true"] {
    background: var(--surface-hover);
    color: var(--text);
  }

  .icon-button:active:not(:disabled, [aria-disabled="true"]) {
    scale: 0.92;
  }

  .icon-button:disabled,
  .icon-button[aria-disabled="true"] {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .busy,
  .busy[aria-disabled="true"] {
    opacity: 1;
    color: var(--text);
    cursor: progress;
  }

  .busy :global(svg) {
    animation: spin 0.9s linear infinite;
  }

  @keyframes spin {
    to {
      rotate: 1turn;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .busy :global(svg) {
      animation: none;
    }
  }
</style>
