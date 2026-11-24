<script lang="ts">
  interface Props {
    checked: boolean;
    label: string;
    description?: string;
    disabled?: boolean;
    onchange: (checked: boolean) => void;
  }

  let { checked, label, description, disabled = false, onchange }: Props = $props();
  const id = $props.id();
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
    aria-describedby={description ? `${id}-description` : undefined}
    {disabled}
    onclick={() => onchange(!checked)}
  >
    <span class="thumb"></span>
  </button>
</div>

<style>
  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    min-height: 40px;
  }

  .text {
    min-width: 0;
  }

  label {
    font-size: var(--fs-md);
    font-weight: 600;
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
    width: 40px;
    height: 22px;
    padding: 0;
    border: 1px solid var(--border-strong);
    border-radius: 999px;
    background: var(--surface-hover);
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-out);
  }

  .thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--text-muted);
    transition: transform var(--duration-base) var(--ease-out);
  }

  .switch[aria-checked="true"] {
    background: var(--accent);
    border-color: var(--accent);
  }

  .switch[aria-checked="true"] .thumb {
    transform: translateX(18px);
    background: var(--accent-contrast);
  }

  .disabled {
    opacity: 0.5;
  }

  .switch:disabled {
    cursor: not-allowed;
  }
</style>
