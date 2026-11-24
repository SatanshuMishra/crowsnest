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
    onpreview?: (value: T, via: "hover" | "focus") => void;
    describedBy?: (value: T) => string | undefined;
  }

  let { label, value, options, description, disabled = false, onchange, onpreview, describedBy }: Props = $props();
  const id = $props.id();

  function onFocus(event: FocusEvent, option: T) {
    if ((event.currentTarget as HTMLElement).matches(":focus-visible")) {
      onpreview?.(option, "focus");
    }
  }

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
        aria-describedby={describedBy?.(option.value)}
        {disabled}
        onclick={() => onchange(option.value)}
        onmouseenter={() => onpreview?.(option.value, "hover")}
        onfocus={(event) => onFocus(event, option.value)}
      >
        {option.label}
      </button>
    {/each}
  </div>
</div>

<style>
  .field {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2) var(--space-5);
    min-height: 40px;
  }

  .text {
    flex: 1 1 12rem;
    min-width: 0;
  }

  .label {
    font-size: var(--fs-md);
  }

  p {
    margin: 2px 0 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .segmented {
    display: inline-flex;
    flex: none;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
  }

  button {
    min-height: var(--target-min);
    padding: 0 var(--space-3);
    border: 0;
    border-radius: 0;
    background: transparent;
    color: var(--text-muted);
    font-size: var(--fs-sm);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }

  button + button {
    border-left: 1px solid var(--border);
  }

  button:first-child {
    border-radius: calc(var(--radius-sm) - 1px) 0 0 calc(var(--radius-sm) - 1px);
  }

  button:last-child {
    border-radius: 0 calc(var(--radius-sm) - 1px) calc(var(--radius-sm) - 1px) 0;
  }

  button:hover:not(:disabled) {
    color: var(--text);
  }

  button[aria-checked="true"] {
    background: var(--accent);
    color: var(--accent-contrast);
  }

  button:disabled {
    cursor: not-allowed;
  }

  .disabled {
    opacity: 0.5;
  }
</style>
