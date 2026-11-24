<script lang="ts">
  import TeamTable from "../monitor/TeamTable.svelte";
  import { markedColumn } from "../../lib/appearance";
  import { SAMPLE_TEAM } from "../../lib/sample";
  import type { Settings } from "../../lib/types";

  interface Props {
    settings: Settings;
    theme: "dark" | "light";
    mark?: boolean;
  }

  let { settings, theme, mark = false }: Props = $props();

  const players = [...SAMPLE_TEAM];
  const marked = $derived(mark ? markedColumn(settings) : null);
</script>

<div
  class="surface"
  data-theme={theme}
  data-palette={settings.palette}
  data-text-size={settings.textSize}
  data-density={settings.density}
  inert
>
  <TeamTable side="allies" title="Sample team" {players} {settings} summary={false} {marked} onopen={() => undefined} />
</div>

<style>
  .surface {
    padding: var(--space-2) var(--space-3) var(--space-3) var(--space-5);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface-raised);
    color: var(--text);
    font-size: var(--fs-md);
  }
</style>
