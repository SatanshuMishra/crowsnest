<script lang="ts">
  import Segmented from "../ui/Segmented.svelte";
  import {
    APPEARANCE,
    isUnavailable,
    optionFor,
    unavailableReason,
    type AppearanceKey,
    type AppearanceOption,
    type AppearanceValues,
  } from "../../lib/appearance";
  import { app } from "../../lib/state.svelte";
  import type { Settings } from "../../lib/types";

  type Value = AppearanceValues[AppearanceKey];

  interface Props {
    key: AppearanceKey;
    settings: Settings;
  }

  let { key, settings }: Props = $props();

  const field = $derived(APPEARANCE[key]);
  const options = $derived(field.options as AppearanceOption<Value>[]);
  const value = $derived<Value>(settings[key]);
  const unavailable = $derived(isUnavailable(key, settings));
  const option = $derived(optionFor(key, value as never));
  const description = $derived.by(() => {
    if (unavailable) {
      return unavailableReason(key) ?? field.description;
    }
    if (key === "theme" && value === "system") {
      return `${option.caption} It is ${app.systemDark ? "dark" : "light"} on this computer.`;
    }
    return option.caption;
  });
  const choices = $derived(options.map((choice) => ({ value: String(choice.value), label: choice.label })));

  function change(next: string) {
    const chosen = options.find((choice) => String(choice.value) === next);
    if (chosen) {
      void app.update({ [key]: chosen.value } as Partial<Settings>);
    }
  }
</script>

<Segmented
  label={field.label}
  {description}
  value={String(value)}
  options={choices}
  disabled={unavailable}
  onchange={change}
/>
