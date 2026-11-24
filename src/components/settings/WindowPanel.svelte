<script lang="ts">
  import Toggle from "../ui/Toggle.svelte";
  import { app } from "../../lib/state.svelte";
  import type { Settings } from "../../lib/types";

  interface Props {
    settings: Settings;
    found: string | null;
  }

  let { settings, found }: Props = $props();
</script>

<div data-setting="alwaysOnTop" data-found={found === "alwaysOnTop" || undefined}>
  <Toggle
    label="Always on top"
    description="Keep Crowsnest above other windows. The pin in the title bar does the same."
    checked={settings.alwaysOnTop}
    onchange={(alwaysOnTop) => app.update({ alwaysOnTop })}
  />
</div>
<div data-setting="closeToTray" data-found={found === "closeToTray" || undefined}>
  <Toggle
    label="Close to tray"
    description="Closing the window hides Crowsnest in the notification area. Quit it from the tray menu."
    checked={settings.closeToTray}
    onchange={(closeToTray) => app.update({ closeToTray })}
  />
</div>
