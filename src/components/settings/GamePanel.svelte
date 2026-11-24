<script lang="ts">
  import InstallPicker from "./InstallPicker.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import { app } from "../../lib/state.svelte";
  import type { Region, Settings } from "../../lib/types";

  interface Props {
    settings: Settings;
    found: string | null;
  }

  let { settings, found }: Props = $props();

  type ServerChoice = "automatic" | Region;

  const regionLabels: Record<Region, string> = { eu: "EU", na: "NA", asia: "Asia" };

  const serverChoices: { value: ServerChoice; label: string }[] = [
    { value: "automatic", label: "Automatic" },
    { value: "eu", label: "EU" },
    { value: "na", label: "NA" },
    { value: "asia", label: "Asia" },
  ];

  const reportedRegion = $derived(app.snapshot?.install?.region ?? null);
  const serverDescription = $derived(
    reportedRegion
      ? `Automatic reads the server from the game, which reports ${regionLabels[reportedRegion]}.`
      : "Automatic reads the server from the game.",
  );

  function chooseServer(choice: ServerChoice) {
    void app.update({ regionOverride: choice === "automatic" ? null : choice });
  }
</script>

<div data-setting="installPath" data-found={found === "installPath" || undefined}>
  <InstallPicker />
</div>
<div data-setting="region" data-found={found === "region" || undefined}>
  <Segmented
    label="Server"
    description={serverDescription}
    value={settings.regionOverride ?? "automatic"}
    options={serverChoices}
    onchange={chooseServer}
  />
</div>
