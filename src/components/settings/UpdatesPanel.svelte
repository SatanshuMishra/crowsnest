<script lang="ts">
  import Button from "../ui/Button.svelte";
  import Icon from "../ui/Icon.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import { app } from "../../lib/state.svelte";
  import type { Settings } from "../../lib/types";
  import { updateAnnouncement, updates, type DownloadingState } from "../../lib/updates.svelte";

  interface Props {
    settings: Settings;
    found: string | null;
  }

  let { settings, found }: Props = $props();

  let offer = $state<HTMLDivElement>();

  const update = $derived(updates.state);
  const updateNews = $derived(updateAnnouncement(update));
  const checkingByRequest = $derived(update.status === "checking" && !update.silent);
  const updateInProgress = $derived(update.status === "downloading" || update.status === "ready");

  function downloadLabel(download: DownloadingState): string {
    if (download.percent !== null) {
      return `${download.percent}%`;
    }
    return download.received === 0 ? "Starting…" : "Size not known";
  }

  function install() {
    offer?.focus();
    void updates.installUpdate();
  }
</script>

<div data-setting="checkForUpdates" data-found={found === "checkForUpdates" || undefined}>
  <Toggle
    label="Check for updates automatically"
    description="Look for a new release a few seconds after Crowsnest starts."
    checked={settings.checkForUpdates}
    onchange={(checkForUpdates) => app.update({ checkForUpdates })}
  />
</div>
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
  {/if}
</div>
<span class="visually-hidden" aria-live="polite">{updateNews}</span>
{#if update.status === "available" || update.status === "downloading" || update.status === "ready"}
  <div bind:this={offer} class="offer" tabindex="-1" role="group" aria-label="Crowsnest update">
    {#if update.status === "available"}
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
        <Button variant="primary" onclick={install}>
          <Icon name="download" />
          {update.error ? "Try again" : `Install ${update.version}`}
        </Button>
      </div>
    {:else if update.status === "downloading"}
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
    {:else if update.status === "ready"}
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
    {/if}
  </div>
{/if}

<style>
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
    font-size: var(--fs-md);
  }

  .label {
    font-size: var(--fs-md);
  }

  .feedback:empty {
    display: none;
  }

  .offer {
    display: grid;
    gap: var(--space-3);
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
    height: 2px;
    overflow: hidden;
    background: var(--border-strong);
  }

  .fill {
    position: absolute;
    inset: 0 auto 0 0;
    width: 0;
    background: var(--text);
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
