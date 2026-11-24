<script lang="ts">
  import PaletteSwatch from "./PaletteSwatch.svelte";
  import {
    APPEARANCE,
    isUnavailable,
    optionFor,
    unavailableReason,
    type AppearanceKey,
    type AppearanceOption,
    type AppearanceValues,
  } from "../../lib/appearance";
  import type { Palette, Settings } from "../../lib/types";

  type Value = AppearanceValues[AppearanceKey];

  interface Props {
    key: AppearanceKey;
    settings: Settings;
    theme: "dark" | "light";
    onchange: (value: Value, option: AppearanceOption<Value>) => void;
  }

  let { key, settings, theme, onchange }: Props = $props();
  const id = $props.id();

  const COLUMNS: Record<AppearanceKey, number> = {
    theme: 3,
    textSize: 3,
    density: 2,
    palette: 1,
    coloredValues: 2,
    highlight: 3,
    highlightStat: 2,
    showClanTags: 2,
  };

  const field = $derived(APPEARANCE[key]);
  const options = $derived(field.options as AppearanceOption<Value>[]);
  const value = $derived<Value>(settings[key]);
  const unavailable = $derived(isUnavailable(key, settings));
  const description = $derived(
    unavailable ? (unavailableReason(key) ?? field.description) : optionFor(key, value as never).caption,
  );

  function onkeydown(event: KeyboardEvent) {
    const step =
      event.key === "ArrowRight" || event.key === "ArrowDown"
        ? 1
        : event.key === "ArrowLeft" || event.key === "ArrowUp"
          ? -1
          : 0;
    const index = options.findIndex((option) => option.value === value);
    if (step === 0 || index < 0 || unavailable) {
      return;
    }
    event.preventDefault();
    const next = options[(index + step + options.length) % options.length];
    if (next) {
      onchange(next.value, next);
      const group = event.currentTarget as HTMLElement;
      requestAnimationFrame(() => {
        group.querySelector<HTMLElement>('[aria-checked="true"]')?.focus();
      });
    }
  }
</script>

<div class="field" class:unavailable>
  <h3 id="{id}-label">{field.label}</h3>
  <div
    class="cards"
    role="radiogroup"
    tabindex="-1"
    aria-labelledby="{id}-label"
    aria-describedby="{id}-description"
    style:--columns={COLUMNS[key]}
    {onkeydown}
  >
    {#each options as option (String(option.value))}
      {@const checked = option.value === value}
      <button
        type="button"
        role="radio"
        class="card"
        aria-checked={checked}
        tabindex={checked ? 0 : -1}
        disabled={unavailable}
        onclick={() => onchange(option.value, option)}
      >
        <span class="dot" aria-hidden="true"></span>
        <span class="label">{option.label}</span>
        {#if key === "palette"}
          <PaletteSwatch palette={option.value as Palette} {theme} />
        {/if}
      </button>
    {/each}
  </div>
  <p id="{id}-description" class="description">{description}</p>
</div>

<style>
  .field {
    display: grid;
    gap: var(--space-2);
  }

  .unavailable {
    opacity: 0.5;
  }

  h3 {
    margin: 0;
    font-size: var(--fs-md);
    font-weight: var(--weight-strong);
  }

  .cards {
    display: grid;
    grid-template-columns: repeat(var(--columns), minmax(0, 1fr));
    gap: var(--space-2);
  }

  .card {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: 36px;
    padding: var(--space-1) var(--space-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
    background: transparent;
    font-size: var(--fs-md);
    text-align: left;
    cursor: pointer;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out);
  }

  .card:hover:not(:disabled) {
    background: var(--surface-hover);
  }

  .card[aria-checked="true"] {
    border-color: var(--text);
    box-shadow: inset 0 0 0 1px var(--text);
    font-weight: var(--weight-strong);
  }

  .card:disabled {
    cursor: not-allowed;
  }

  .dot {
    flex: none;
    width: 14px;
    height: 14px;
    border: 1.5px solid var(--text-muted);
    border-radius: 50%;
  }

  .card[aria-checked="true"] .dot {
    border-color: var(--text);
    background: var(--text);
    box-shadow: inset 0 0 0 2.5px var(--surface-raised);
  }

  .label {
    flex: 1;
    min-width: 0;
  }

  .description {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }
</style>
