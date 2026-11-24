<script lang="ts">
  import Button from "../ui/Button.svelte";
  import Icon, { type IconName } from "../ui/Icon.svelte";
  import { app } from "../../lib/state.svelte";
  import type { InstallSource, NoticeLevel, Settings, Snapshot } from "../../lib/types";

  interface Props {
    snapshot: Snapshot;
    settings: Settings;
    battleShown: boolean;
  }

  let { snapshot, settings, battleShown }: Props = $props();
  const id = $props.id();

  const NOTICE_ICONS: Record<NoticeLevel, IconName> = {
    info: "info",
    warning: "warning",
    error: "error",
  };

  const NOTICE_NAMES: Record<NoticeLevel, string> = {
    info: "Note",
    warning: "Warning",
    error: "Error",
  };

  const SOURCES: Record<InstallSource, string> = {
    gameCenter: "found through Wargaming Game Center",
    steam: "found through Steam",
    defaultPath: "found in the default location",
    manual: "chosen in Settings",
  };

  const install = $derived(snapshot.install);
  const replaysDir = $derived(settings.replaysDirOverride ?? install?.replaysDir ?? null);

  function openSettings() {
    app.view = "settings";
  }
</script>

{#if snapshot.notice}
  <div class="notice {snapshot.notice.level}" role={snapshot.notice.level === "error" ? "alert" : "status"}>
    <Icon name={NOTICE_ICONS[snapshot.notice.level]} />
    <p><span class="visually-hidden">{NOTICE_NAMES[snapshot.notice.level]}: </span>{snapshot.notice.message}</p>
  </div>
{/if}

{#if !battleShown}
  <section class="state" aria-labelledby="{id}-title">
    {#if snapshot.phase === "replaysDisabled"}
      <div class="panel">
        <h2 id="{id}-title">Replays are turned off</h2>
        <p>
          Crowsnest reads each battle from a file the game writes into its replays folder when a battle loads. With
          replays off the game never writes that file, so there is nothing to show.
        </p>
        <h3>Turn replays on</h3>
        <ol>
          <li>Close the game.</li>
          <li>
            In the game folder, open <code>bin\&lt;build&gt;\res_mods\engine_config.xml</code>, or
            <code>bin\&lt;build&gt;\res\engine_config.xml</code> when there is no copy in res_mods.
          </li>
          <li>
            Inside <code>&lt;replays&gt;</code>, change <code>&lt;isEnabled&gt;false&lt;/isEnabled&gt;</code> to
            <code>true</code>, or delete that line.
          </li>
          <li>Start the game, then restart Crowsnest.</li>
        </ol>
        {#if install}
          <p class="path-line">Game folder: <code>{install.root}</code></p>
        {/if}
        <p>If the game already records replays somewhere else, choose that folder in Settings.</p>
        <div class="actions">
          <Button variant="primary" onclick={openSettings}>
            <Icon name="folder" />
            Choose a replays folder
          </Button>
        </div>
      </div>
    {:else if snapshot.phase === "loading"}
      <div class="panel" aria-busy="true">
        <h2 id="{id}-title">Loading the battle</h2>
        <p>Reading the roster and asking Wargaming for statistics.</p>
        <div class="skeletons" aria-hidden="true">
          <span class="bar"></span>
          <span class="bar"></span>
          <span class="bar"></span>
        </div>
      </div>
    {:else if snapshot.phase === "waiting"}
      <div class="panel">
        <p class="link" class:connected={snapshot.connected}>
          <span class="dot" aria-hidden="true"></span>
          {snapshot.connected ? "Connected to World of Warships" : "Not connected to World of Warships"}
        </p>
        <h2 id="{id}-title">Waiting for a battle</h2>
        <p>
          {snapshot.connected
            ? "The player table appears here as soon as a battle loads."
            : "Crowsnest connects as soon as it can watch the replays folder."}
        </p>
        <dl class="paths">
          <div>
            <dt>Game folder</dt>
            <dd>
              {#if install}
                <code>{install.root}</code>
                <span class="source">{SOURCES[install.source]}</span>
              {:else}
                <span class="source">Not detected</span>
              {/if}
            </dd>
          </div>
          <div>
            <dt>Replays folder</dt>
            <dd>
              {#if replaysDir}
                <code>{replaysDir}</code>
                {#if settings.replaysDirOverride}<span class="source">chosen in Settings</span>{/if}
                {#if snapshot.connected}<span class="source">watching for new battles</span>{/if}
              {:else}
                <span class="source">Not detected</span>
              {/if}
            </dd>
          </div>
        </dl>
      </div>
    {:else}
      <div class="panel">
        <h2 id="{id}-title">Crowsnest needs setting up</h2>
        <p>Choose the game folder and the Wargaming application id before the monitor can start.</p>
        <div class="actions">
          <Button variant="primary" onclick={openSettings}>Open settings</Button>
        </div>
      </div>
    {/if}
  </section>
{/if}

<style>
  .notice {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--border);
    color: var(--text);
    font-size: var(--fs-md);
  }

  .notice :global(svg) {
    flex: none;
    margin-top: 0.125em;
    color: var(--notice-tone);
  }

  .notice p {
    margin: 0;
  }

  .info {
    --notice-tone: var(--info);
  }

  .warning {
    --notice-tone: var(--warning);
  }

  .error {
    --notice-tone: var(--danger);
  }

  .state {
    display: grid;
    flex: 1;
    place-items: center;
    padding: var(--space-6) 0 calc(var(--space-6) * 2);
  }

  .panel {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-3);
    width: min(100%, var(--measure));
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--fs-xl);
    font-weight: var(--weight-strong);
    letter-spacing: var(--tracking-display);
  }

  h3 {
    margin: var(--space-2) 0 0;
    font-size: var(--fs-md);
    font-weight: var(--weight-strong);
  }

  p,
  ol {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-md);
  }

  ol {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding-left: var(--space-5);
  }

  li::marker {
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
  }

  code {
    color: var(--text);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    overflow-wrap: anywhere;
  }

  .path-line {
    overflow-wrap: anywhere;
  }

  .actions {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }

  .paths {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    gap: var(--space-2) var(--space-5);
    width: 100%;
    margin: var(--space-3) 0 0;
    padding-top: var(--space-4);
    border-top: 1px solid var(--border);
  }

  .paths div {
    display: contents;
  }

  dt {
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  dd {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0 var(--space-2);
    margin: 0;
    min-width: 0;
  }

  .source {
    color: var(--text-faint);
    font-size: var(--fs-sm);
  }

  .link {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    margin: 0;
    color: var(--warning);
    font-size: var(--fs-sm);
    font-weight: var(--weight-medium);
  }

  .link.connected {
    color: var(--text);
  }

  .dot {
    flex: none;
    width: var(--space-2);
    height: var(--space-2);
    border-radius: 50%;
    background: var(--warning);
  }

  .connected .dot {
    background: var(--success);
  }

  .skeletons {
    display: grid;
    gap: var(--space-2);
    width: 100%;
    margin-top: var(--space-2);
  }

  .bar {
    height: var(--row-height);
    border-radius: 2px;
    background: var(--surface-hover);
    animation: pulse calc(var(--duration-base) * 4) var(--ease-out) infinite alternate;
  }

  @keyframes pulse {
    from {
      opacity: 1;
    }

    to {
      opacity: 0.45;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .bar {
      animation: none;
    }
  }
</style>
