<script lang="ts">
  import AppearanceSetting from "./AppearanceSetting.svelte";
  import SettingsGroup from "./SettingsGroup.svelte";
  import Button from "../ui/Button.svelte";
  import type { AppearanceKey } from "../../lib/appearance";
  import { preview } from "../../lib/preview.svelte";
  import { app } from "../../lib/state.svelte";
  import type { Settings } from "../../lib/types";

  interface Props {
    settings: Settings;
    found: string | null;
  }

  let { settings, found }: Props = $props();
  const id = $props.id();

  const GROUPS: readonly (readonly [string, readonly AppearanceKey[]])[] = [
    ["Theme and size", ["theme", "textSize", "density"]],
    ["Colour", ["palette", "coloredValues"]],
    ["Rows", ["highlight", "highlightStat", "showClanTags"]],
  ];

  function openTour() {
    preview.clear();
    app.setup = "tour";
  }
</script>

{#each GROUPS as [title, keys] (title)}
  <SettingsGroup {title}>
    {#each keys as key (key)}
      <AppearanceSetting {key} {settings} found={found === key} />
    {/each}
  </SettingsGroup>
{/each}

<SettingsGroup title="Tour">
  <div class="tour" data-setting="tour" data-found={found === "tour" || undefined}>
    <div>
      <p class="label">Appearance tour</p>
      <p class="description" id="{id}-description">Walk through the appearance options with a live preview.</p>
    </div>
    <Button id="appearance-tour" aria-describedby="{id}-description" onclick={openTour}>Take the appearance tour</Button>
  </div>
</SettingsGroup>

<style>
  .tour {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2) var(--space-5);
    min-height: 40px;
  }

  p {
    margin: 0;
  }

  .label {
    font-size: var(--fs-md);
  }

  .description {
    margin-top: 2px;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }
</style>
