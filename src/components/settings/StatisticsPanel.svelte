<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import ApplicationIdField from "./ApplicationIdField.svelte";
  import Icon from "../ui/Icon.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import { app } from "../../lib/state.svelte";
  import type { Settings, TeamAverage } from "../../lib/types";

  interface Props {
    settings: Settings;
    found: string | null;
  }

  let { settings, found }: Props = $props();

  const DEVELOPERS_URL = "https://developers.wargaming.net/applications/";

  const teamAverageChoices: { value: TeamAverage; label: string; explanation: string }[] = [
    { value: "mean", label: "Mean", explanation: "every player counts the same." },
    {
      value: "weighted",
      label: "Weighted",
      explanation: "win rate weighted by battles played, with each player capped at 40 battles.",
    },
    { value: "median", label: "Median", explanation: "the middle player, so one outlier cannot skew the team." },
  ];

  let linkError = $state<string | null>(null);

  async function openDevelopers(event: MouseEvent) {
    event.preventDefault();
    linkError = null;
    try {
      await openUrl(DEVELOPERS_URL);
    } catch {
      linkError = `Your browser could not be opened. The address is ${DEVELOPERS_URL}`;
    }
  }
</script>

<p class="hint">
  Get a free application ID by creating a Standalone application on
  <a href={DEVELOPERS_URL} onclick={openDevelopers}>developers.wargaming.net</a>.
</p>
<div class="feedback" aria-live="polite">
  {#if linkError}
    <p class="message error"><Icon name="error" /><span>{linkError}</span></p>
  {/if}
</div>
<div data-setting="applicationId" data-found={found === "applicationId" || undefined}>
  <ApplicationIdField />
</div>
<div data-setting="soloStats" data-found={found === "soloStats" || undefined}>
  <Toggle
    label="Solo statistics"
    description="Count only battles played without a division."
    checked={settings.soloStats}
    onchange={(soloStats) => app.update({ soloStats })}
  />
</div>
<div data-setting="teamAverage" data-found={found === "teamAverage" || undefined}>
  <Segmented
    label="Team average"
    description="How the team summary combines its players."
    value={settings.teamAverage}
    options={teamAverageChoices}
    onchange={(teamAverage) => app.update({ teamAverage })}
  />
  <dl class="explanations">
    {#each teamAverageChoices as choice (choice.value)}
      <div>
        <dt>{choice.label}:</dt>
        <dd>{choice.explanation}</dd>
      </div>
    {/each}
  </dl>
</div>

<style>
  .hint {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  a {
    color: var(--text);
    text-decoration: underline;
    text-decoration-color: var(--text-faint);
    text-underline-offset: 0.2em;
  }

  a:hover {
    text-decoration-color: currentColor;
  }

  .feedback:empty {
    display: none;
  }

  .message {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    margin: 0;
    font-size: var(--fs-md);
  }

  .message :global(svg) {
    flex: none;
    margin-top: 0.125em;
  }

  .error {
    color: var(--danger);
  }

  .explanations {
    display: grid;
    gap: var(--space-1);
    margin: 0 0 var(--space-2);
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .explanations div {
    display: flex;
    gap: var(--space-1);
  }

  .explanations dt {
    color: var(--text);
  }

  .explanations dd {
    margin: 0;
  }
</style>
