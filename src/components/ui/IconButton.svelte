<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";
  import Icon, { type IconName } from "./Icon.svelte";

  interface Props extends HTMLButtonAttributes {
    icon: IconName;
    label: string;
    pressed?: boolean;
  }

  let { icon, label, pressed, disabled = false, type = "button", ...rest }: Props = $props();
</script>

<button
  {...rest}
  {type}
  class="icon-button"
  aria-label={label}
  title={label}
  aria-pressed={pressed}
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
      color var(--duration-fast) var(--ease-out);
  }

  .icon-button:hover:not(:disabled),
  .icon-button[aria-pressed="true"] {
    background: var(--surface-hover);
    color: var(--text);
  }

  .icon-button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
</style>
