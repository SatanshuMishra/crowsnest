<script lang="ts">
  interface Props {
    checked: boolean;
    label: string;
    description?: string;
    disabled?: boolean;
    onchange: (checked: boolean) => void;
    onpreview?: (via: "hover" | "focus") => void;
    describedBy?: string;
  }

  let { checked, label, description, disabled = false, onchange, onpreview, describedBy }: Props = $props();
  const id = $props.id();
  const describers = $derived(
    [description ? `${id}-description` : null, describedBy ?? null].filter((part) => part !== null).join(" ") ||
      undefined,
  );

  function onFocus(event: FocusEvent) {
    if ((event.currentTarget as HTMLElement).matches(":focus-visible")) {
      onpreview?.("focus");
    }
  }
</script>

<div class="toggle-row" class:disabled>
  <div class="text">
    <label for={id} id="{id}-label">{label}</label>
    {#if description}
      <p id="{id}-description">{description}</p>
    {/if}
  </div>
  <button
    {id}
    type="button"
    role="switch"
    class="switch"
    aria-checked={checked}
    aria-labelledby="{id}-label"
    aria-describedby={describers}
    {disabled}
    onclick={() => onchange(!checked)}
    onmouseenter={() => onpreview?.("hover")}
    onfocus={onFocus}
  >
    <span class="thumb"></span>
  </button>
</div>

<style>
  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-5);
    min-height: 40px;
  }

  .text {
    min-width: 0;
  }

  label {
    font-size: var(--fs-md);
    cursor: pointer;
  }

  p {
    margin: 2px 0 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .switch {
    position: relative;
    flex: none;
    width: 36px;
    height: 20px;
    padding: 0;
    border: 1px solid var(--text-faint);
    border-radius: 999px;
    background: transparent;
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out);
  }

  .switch::before {
    position: absolute;
    inset: -5px -4px;
    content: "";
  }

  .thumb {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--text-muted);
    transition: transform var(--duration-base) var(--ease-out);
  }

  .switch[aria-checked="true"] {
    border-color: var(--accent);
    background: var(--accent);
  }

  .switch[aria-checked="true"] .thumb {
    transform: translateX(16px);
    background: var(--accent-contrast);
  }

  .disabled {
    opacity: 0.5;
  }

  .switch:disabled {
    cursor: not-allowed;
  }
</style>
