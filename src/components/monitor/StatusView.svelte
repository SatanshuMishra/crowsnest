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
        <span class="mark warning"><Icon name="warning" size={24} /></span>
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
      <div class="panel calm">
        <span class="mark"><Icon name="anchor" size={24} /></span>
        <h2 id="{id}-title">Waiting for a battle</h2>
        <p>The player table appears here as soon as a battle loads.</p>
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
              {:else}
                <span class="source">Not detected</span>
              {/if}
            </dd>
          </div>
        </dl>
      </div>
    {:else}
      <div class="panel">
        <span class="mark warning"><Icon name="settings" size={24} /></span>
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
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--notice-tone);
    border-left-width: 3px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--notice-tone) var(--highlight-fill-alpha), var(--surface));
    color: var(--text);
    font-size: var(--fs-md);
  }

  .notice :global(svg) {
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
    padding: var(--space-6) var(--space-4);
  }

  .panel {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-3);
    width: min(100%, calc(var(--space-6) * 18));
    padding: var(--space-5);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface);
  }

  .calm {
    align-items: center;
    text-align: center;
  }

  .mark {
    display: inline-grid;
    place-items: center;
    color: var(--accent);
  }

  .mark.warning {
    color: var(--warning);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--fs-xl);
    font-weight: 600;
  }

  h3 {
    margin: 0;
    font-size: var(--fs-md);
    font-weight: 600;
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
    gap: var(--space-1);
    padding-left: var(--space-5);
  }

  code {
    padding: 0 var(--space-1);
    border-radius: var(--radius-sm);
    background: var(--surface-hover);
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
  }

  .paths {
    display: grid;
    gap: var(--space-2);
    width: 100%;
    margin: var(--space-2) 0 0;
    text-align: left;
  }

  .paths div {
    display: grid;
    gap: var(--space-1);
  }

  dt {
    color: var(--text-muted);
    font-size: var(--fs-xs);
    font-weight: 600;
    text-transform: uppercase;
  }

  dd {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--space-2);
    margin: 0;
  }

  .source {
    color: var(--text-faint);
    font-size: var(--fs-sm);
  }

  .skeletons {
    display: grid;
    gap: var(--space-2);
    width: 100%;
  }

  .bar {
    height: var(--row-height);
    border-radius: var(--radius-sm);
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
