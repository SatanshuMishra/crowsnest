<script lang="ts" generics="T extends string">
  interface Option {
    value: T;
    label: string;
  }

  interface Props {
    label: string;
    value: T;
    options: Option[];
    description?: string;
    disabled?: boolean;
    onchange: (value: T) => void;
  }

  let { label, value, options, description, disabled = false, onchange }: Props = $props();
  const id = $props.id();

  function onKeydown(event: KeyboardEvent) {
    const index = options.findIndex((option) => option.value === value);
    const step = event.key === "ArrowRight" || event.key === "ArrowDown" ? 1 : event.key === "ArrowLeft" || event.key === "ArrowUp" ? -1 : 0;
    if (step === 0 || index < 0) {
      return;
    }
    event.preventDefault();
    const next = options[(index + step + options.length) % options.length];
    if (next) {
      onchange(next.value);
      const group = event.currentTarget as HTMLElement;
      requestAnimationFrame(() => {
        group.querySelector<HTMLElement>('[aria-checked="true"]')?.focus();
      });
    }
  }
</script>

<div class="field" class:disabled>
  <div class="text">
    <span class="label" id="{id}-label">{label}</span>
    {#if description}
      <p id="{id}-description">{description}</p>
    {/if}
  </div>
  <div
    class="segmented"
    role="radiogroup"
    tabindex="-1"
    aria-labelledby="{id}-label"
    aria-describedby={description ? `${id}-description` : undefined}
    onkeydown={onKeydown}
  >
    {#each options as option (option.value)}
      <button
        type="button"
        role="radio"
        aria-checked={option.value === value}
        tabindex={option.value === value ? 0 : -1}
        {disabled}
        onclick={() => onchange(option.value)}
      >
        {option.label}
      </button>
    {/each}
  </div>
</div>

<style>
  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    min-height: 40px;
  }

  .label {
    font-size: var(--fs-md);
    font-weight: 600;
  }

  p {
    margin: 2px 0 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .segmented {
    display: inline-flex;
    flex: none;
    padding: 2px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
  }

  button {
    min-height: var(--target-min);
    padding: 0 var(--space-3);
    border: 0;
    border-radius: 3px;
    background: transparent;
    color: var(--text-muted);
    font-size: var(--fs-sm);
    font-weight: 600;
    cursor: pointer;
  }

  button[aria-checked="true"] {
    background: var(--surface-raised);
    color: var(--text);
    box-shadow: inset 0 0 0 1px var(--border-strong);
  }

  button:disabled {
    cursor: not-allowed;
  }

  .disabled {
    opacity: 0.5;
  }
</style>
