<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { onMount } from "svelte";
  import About from "../components/settings/About.svelte";
  import ApplicationIdField from "../components/settings/ApplicationIdField.svelte";
  import InstallPicker from "../components/settings/InstallPicker.svelte";
  import SettingsSection from "../components/settings/SettingsSection.svelte";
  import Button from "../components/ui/Button.svelte";
  import Icon from "../components/ui/Icon.svelte";
  import IconButton from "../components/ui/IconButton.svelte";
  import Segmented from "../components/ui/Segmented.svelte";
  import Toggle from "../components/ui/Toggle.svelte";
  import { app } from "../lib/state.svelte";
  import type { Density, Highlight, HighlightStat, Region, TeamAverage, TextSize, Theme } from "../lib/types";
  import { updates, type DownloadingState } from "../lib/updates.svelte";

  interface Choice<T extends string> {
    value: T;
    label: string;
  }

  type ServerChoice = "automatic" | Region;

  const DEVELOPERS_URL = "https://developers.wargaming.net/applications/";

  const regionLabels: Record<Region, string> = { eu: "EU", na: "NA", asia: "Asia" };

  const serverChoices: Choice<ServerChoice>[] = [
    { value: "automatic", label: "Automatic" },
    { value: "eu", label: "EU" },
    { value: "na", label: "NA" },
    { value: "asia", label: "Asia" },
  ];

  const teamAverageChoices: (Choice<TeamAverage> & { explanation: string })[] = [
    { value: "mean", label: "Mean", explanation: "every player counts the same." },
    {
      value: "weighted",
      label: "Weighted",
      explanation: "win rate weighted by battles played, with each player capped at 40 battles.",
    },
    { value: "median", label: "Median", explanation: "the middle player, so one outlier cannot skew the team." },
  ];

  const themeChoices: Choice<Theme>[] = [
    { value: "system", label: "System" },
    { value: "dark", label: "Dark" },
    { value: "light", label: "Light" },
  ];

  const densityChoices: Choice<Density>[] = [
    { value: "comfortable", label: "Comfortable" },
    { value: "compact", label: "Compact" },
  ];

  const textSizeChoices: Choice<TextSize>[] = [
    { value: "small", label: "Small" },
    { value: "normal", label: "Normal" },
    { value: "large", label: "Large" },
  ];

  const highlightChoices: Choice<Highlight>[] = [
    { value: "none", label: "None" },
    { value: "edge", label: "Edge" },
    { value: "fill", label: "Fill" },
  ];

  const highlightStatChoices: Choice<HighlightStat>[] = [
    { value: "personalRating", label: "Personal rating" },
    { value: "shipWinRate", label: "Ship win rate" },
    { value: "accountWinRate", label: "Account win rate" },
    { value: "averageDamage", label: "Average damage" },
  ];

  let heading = $state<HTMLHeadingElement>();
  let linkError = $state<string | null>(null);

  const settings = $derived(app.settings);
  const update = $derived(updates.state);
  const reportedRegion = $derived(app.snapshot?.install?.region ?? null);
  const serverDescription = $derived(
    reportedRegion
      ? `Automatic reads the server from the game, which reports ${regionLabels[reportedRegion]}.`
      : "Automatic reads the server from the game.",
  );
  const checkingByRequest = $derived(update.status === "checking" && !update.silent);
  const updateInProgress = $derived(update.status === "downloading" || update.status === "ready");

  onMount(() => {
    heading?.focus();
  });

  function downloadLabel(download: DownloadingState): string {
    if (download.percent !== null) {
      return `${download.percent}%`;
    }
    return download.received === 0 ? "Starting…" : "Size not known";
  }

  function chooseServer(choice: ServerChoice) {
    void app.update({ regionOverride: choice === "automatic" ? null : choice });
  }

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

<main class="settings">
  <div class="column">
    <header class="header">
      <IconButton icon="back" label="Back" onclick={() => (app.view = "monitor")} />
      <h1 tabindex="-1" bind:this={heading}>Settings</h1>
    </header>

    {#if app.saveError}
      <p class="save-error" role="alert">
        <Icon name="error" />
        <span>Your last change was not saved: {app.saveError}</span>
      </p>
    {/if}

    {#if settings}
      <SettingsSection title="Game" description="Where Crowsnest finds the game, its replays and your server.">
        <InstallPicker />
        <Segmented
          label="Server"
          description={serverDescription}
          value={settings.regionOverride ?? "automatic"}
          options={serverChoices}
          onchange={chooseServer}
        />
      </SettingsSection>

      <SettingsSection title="Statistics" description="Crowsnest reads public statistics from the Wargaming API.">
        <p class="hint">
          Get a free application ID by creating a Standalone application on
          <a href={DEVELOPERS_URL} onclick={openDevelopers}>developers.wargaming.net</a>.
        </p>
        <div class="feedback" aria-live="polite">
          {#if linkError}
            <p class="message error"><Icon name="error" /><span>{linkError}</span></p>
          {/if}
        </div>
        <ApplicationIdField />
        <Toggle
          label="Solo statistics"
          description="Count only battles played without a division."
          checked={settings.soloStats}
          onchange={(soloStats) => app.update({ soloStats })}
        />
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
      </SettingsSection>

      <SettingsSection title="Appearance">
        <Segmented
          label="Theme"
          description="System follows your Windows setting."
          value={settings.theme}
          options={themeChoices}
          onchange={(theme) => app.update({ theme })}
        />
        <Segmented
          label="Density"
          value={settings.density}
          options={densityChoices}
          onchange={(density) => app.update({ density })}
        />
        <Segmented
          label="Text size"
          value={settings.textSize}
          options={textSizeChoices}
          onchange={(textSize) => app.update({ textSize })}
        />
        <Toggle
          label="Coloured values"
          description="Colour each statistic by its tier. The tier name stays available either way."
          checked={settings.coloredValues}
          onchange={(coloredValues) => app.update({ coloredValues })}
        />
        <Segmented
          label="Row highlight"
          description="Mark each row with a coloured left edge or a tinted fill."
          value={settings.highlight}
          options={highlightChoices}
          onchange={(highlight) => app.update({ highlight })}
        />
        <Segmented
          label="Highlight by"
          description={settings.highlight === "none"
            ? "Unavailable while Row highlight is None. Choose Edge or Fill first."
            : "The statistic whose tier sets each row's highlight."}
          disabled={settings.highlight === "none"}
          value={settings.highlightStat}
          options={highlightStatChoices}
          onchange={(highlightStat) => app.update({ highlightStat })}
        />
        <Toggle
          label="Show clan tags"
          description="Show each player's clan tag next to their name."
          checked={settings.showClanTags}
          onchange={(showClanTags) => app.update({ showClanTags })}
        />
      </SettingsSection>

      <SettingsSection title="Window">
        <Toggle
          label="Always on top"
          description="Keep Crowsnest above other windows."
          checked={settings.alwaysOnTop}
          onchange={(alwaysOnTop) => app.update({ alwaysOnTop })}
        />
        <Toggle
          label="Close to tray"
          description="Closing the window hides Crowsnest in the notification area. Quit it from the tray menu."
          checked={settings.closeToTray}
          onchange={(closeToTray) => app.update({ closeToTray })}
        />
      </SettingsSection>

      <SettingsSection title="Updates" description="New releases come from the Crowsnest page on GitHub.">
        <Toggle
          label="Check for updates automatically"
          description="Look for a new release a few seconds after Crowsnest starts."
          checked={settings.checkForUpdates}
          onchange={(checkForUpdates) => app.update({ checkForUpdates })}
        />
        <div class="version-row">
          <div class="version-text">
            <span class="label">Installed version</span>
            <span class="num">{app.info?.version ?? "Not known"}</span>
          </div>
          <Button
            busy={checkingByRequest}
            disabled={updateInProgress}
            title={updateInProgress ? "An update is already being installed" : undefined}
            onclick={() => updates.checkNow()}
          >
            <Icon name="refresh" />
            {checkingByRequest ? "Checking…" : "Check now"}
          </Button>
        </div>
        <div class="feedback" aria-live="polite">
          {#if checkingByRequest}
            <p class="message muted"><Icon name="refresh" /><span>Checking for updates…</span></p>
          {:else if update.status === "upToDate"}
            <p class="message success"><Icon name="check" /><span>Crowsnest is up to date.</span></p>
          {:else if update.status === "error"}
            <p class="message error"><Icon name="error" /><span>{update.message}</span></p>
          {:else if update.status === "available"}
            <div class="offer">
              <p class="message">
                <Icon name="download" />
                <span>Version <span class="num">{update.version}</span> is available.</span>
              </p>
              {#if update.error}
                <p class="message error"><Icon name="error" /><span>{update.error}</span></p>
              {/if}
              {#if update.notes}
                <details class="notes">
                  <summary>What's new</summary>
                  <p>{update.notes}</p>
                </details>
              {/if}
              <div class="actions">
                <Button variant="primary" onclick={() => updates.installUpdate()}>
                  <Icon name="download" />
                  {update.error ? "Try again" : `Install ${update.version}`}
                </Button>
              </div>
            </div>
          {:else if update.status === "downloading"}
            <div class="offer">
              <p class="message">
                <Icon name="download" />
                <span>Downloading version <span class="num">{update.version}</span>…</span>
              </p>
              <div class="progress">
                <div
                  class="bar"
                  class:indeterminate={update.percent === null}
                  role="progressbar"
                  aria-label="Update download"
                  aria-valuemin={0}
                  aria-valuemax={100}
                  aria-valuenow={update.percent ?? undefined}
                >
                  <span class="fill" style:width={update.percent === null ? undefined : `${update.percent}%`}></span>
                </div>
                <span class="num">{downloadLabel(update)}</span>
              </div>
            </div>
          {:else if update.status === "ready"}
            <div class="offer">
              <p class="message success">
                <Icon name="check" />
                <span>Version <span class="num">{update.version}</span> is installed. Restart Crowsnest to use it.</span>
              </p>
              {#if update.error}
                <p class="message error"><Icon name="error" /><span>{update.error}</span></p>
              {/if}
              <div class="actions">
                <Button variant="primary" onclick={() => updates.restart()}>Restart to update</Button>
              </div>
            </div>
          {/if}
        </div>
      </SettingsSection>

      <SettingsSection title="About">
        <About />
      </SettingsSection>
    {/if}
  </div>
</main>

<style>
  .settings {
    height: 100%;
    overflow-y: auto;
    padding: var(--space-4) var(--space-4) var(--space-6);
  }

  .column {
    display: grid;
    width: 100%;
    max-width: calc(var(--space-6) * 22.5);
    margin: 0 auto;
  }

  .header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding-bottom: var(--space-3);
  }

  h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--fs-xl);
    font-weight: 600;
  }

  .save-error {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    margin: 0 0 var(--space-4);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--danger);
    border-radius: var(--radius-md);
    color: var(--danger);
    font-size: var(--fs-md);
  }

  .hint {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  a {
    color: var(--accent);
    font-weight: 600;
    text-decoration: underline;
  }

  a:hover {
    color: var(--accent-strong);
  }

  .explanations {
    display: grid;
    gap: var(--space-1);
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .explanations div {
    display: flex;
    gap: var(--space-1);
  }

  .explanations dt {
    color: var(--text);
    font-weight: 600;
  }

  .explanations dd {
    margin: 0;
  }

  .version-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2) var(--space-4);
    min-height: var(--target-min);
  }

  .version-text {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
  }

  .label {
    font-size: var(--fs-md);
    font-weight: 600;
  }

  .feedback:empty {
    display: none;
  }

  .offer {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface);
  }

  .message {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    margin: 0;
    font-size: var(--fs-md);
  }

  .muted {
    color: var(--text-muted);
  }

  .success {
    color: var(--success);
  }

  .error {
    color: var(--danger);
  }

  .notes summary {
    min-height: var(--target-min);
    line-height: var(--target-min);
    color: var(--text-muted);
    font-size: var(--fs-sm);
    font-weight: 600;
    cursor: pointer;
  }

  .notes p {
    max-height: calc(var(--space-6) * 5);
    margin: 0;
    overflow-y: auto;
    color: var(--text-muted);
    font-size: var(--fs-sm);
    white-space: pre-wrap;
  }

  .actions {
    display: flex;
    gap: var(--space-2);
  }

  .progress {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    font-size: var(--fs-sm);
  }

  .bar {
    position: relative;
    flex: 1 1 auto;
    height: var(--space-2);
    overflow: hidden;
    border-radius: var(--radius-sm);
    background: var(--border-strong);
  }

  .fill {
    position: absolute;
    inset: 0 auto 0 0;
    width: 0;
    background: var(--accent);
    transition: width var(--duration-base) var(--ease-out);
  }

  .indeterminate .fill {
    width: 40%;
    animation: sweep calc(var(--duration-base) * 6) var(--ease-out) infinite;
  }

  @keyframes sweep {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(250%);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .indeterminate .fill {
      animation: none;
    }
  }
</style>
