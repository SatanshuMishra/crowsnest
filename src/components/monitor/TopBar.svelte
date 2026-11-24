<script lang="ts">
  import UpdateNotice from "../UpdateNotice.svelte";
  import IconButton from "../ui/IconButton.svelte";
  import { backend } from "../../lib/backend";
  import { focusViewHeading } from "../../lib/focus";
  import { mapName, matchType, regionLabel, regionName } from "../../lib/format";
  import { app } from "../../lib/state.svelte";
  import type { Phase, Snapshot } from "../../lib/types";

  interface Props {
    snapshot: Snapshot;
  }

  let { snapshot }: Props = $props();

  type Tone = "idle" | "busy" | "live" | "attention";

  const STATUS: Record<Phase, { label: string; tone: Tone }> = {
    waiting: { label: "Waiting for battle", tone: "idle" },
    loading: { label: "Loading stats", tone: "busy" },
    inBattle: { label: "In battle", tone: "live" },
    replaysDisabled: { label: "Replays off", tone: "attention" },
    needsInstall: { label: "Setup needed", tone: "attention" },
    needsApplicationId: { label: "Setup needed", tone: "attention" },
  };

  let refreshing = $state(false);
  let refreshError = $state<string | null>(null);

  const battle = $derived(snapshot.battle);
  const region = $derived(battle?.region ?? snapshot.install?.region ?? null);
  const lastBattle = $derived(battle !== null && snapshot.phase === "waiting");
  const status = $derived(STATUS[snapshot.phase]);

  async function refresh() {
    refreshing = true;
    refreshError = null;
    try {
      await backend.refresh();
    } catch (error) {
      refreshError = String(error);
    } finally {
      refreshing = false;
    }
  }

  function openSettings() {
    app.view = "settings";
  }
</script>

<header class="top-bar">
  <svg class="logo" viewBox="0 0 24 24" width="22" height="22" aria-hidden="true" focusable="false">
    <circle cx="12" cy="12" r="9.5" fill="none" stroke="currentColor" stroke-width="1.6" />
    <circle cx="12" cy="12" r="5.25" fill="none" stroke="currentColor" stroke-width="1.2" opacity="0.55" />
    <path d="M12 12 12 2.5A9.5 9.5 0 0 1 20.23 7.25Z" fill="currentColor" opacity="0.3" />
    <path d="M12 12 20.23 7.25" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
    <circle cx="14.6" cy="7.6" r="1.2" fill="currentColor" />
  </svg>

  <div class="match">
    <h1 tabindex="-1" {@attach focusViewHeading}>
      {#if battle}
        <span class="visually-hidden">Crowsnest, </span>{mapName(battle.arena.mapDisplayName, battle.arena.mapName)}
      {:else}
        Crowsnest
      {/if}
    </h1>
    {#if battle}
      <span class="mode">{matchType(battle.arena.gameType, battle.arena.matchGroup)}</span>
      {#if lastBattle}
        <span class="tag">Last battle</span>
      {/if}
    {/if}
    {#if region}
      <span class="chip" title="Server region: {regionName(region)}">
        <span class="visually-hidden">Server region </span>{regionLabel(region)}
      </span>
    {/if}
  </div>

  <p class="status {status.tone}" aria-live="polite" aria-atomic="true">
    <span class="dot" aria-hidden="true"></span>{status.label}
  </p>

  <div class="actions">
    <UpdateNotice />
    {#if refreshError}
      <span class="refresh-error" role="alert" title={refreshError}>Refresh failed</span>
    {/if}
    <IconButton
      icon="refresh"
      label={battle ? "Refresh statistics" : "Refresh statistics, available once a battle loads"}
      disabled={!battle || refreshing}
      onclick={refresh}
    />
    <IconButton icon="settings" label="Settings" onclick={openSettings} />
  </div>
</header>

<style>
  .top-bar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-height: calc(var(--target-min) + var(--space-4));
    padding: var(--space-1) var(--space-2) var(--space-1) var(--space-3);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .logo {
    flex: none;
    color: var(--accent);
  }

  .match {
    display: flex;
    flex: 1;
    align-items: baseline;
    gap: var(--space-2);
    min-width: 0;
    overflow: clip;
    overflow-clip-margin: var(--space-1);
    white-space: nowrap;
  }

  h1 {
    margin: 0;
    overflow: hidden;
    font-family: var(--font-display);
    font-size: var(--fs-lg);
    font-weight: 600;
    text-overflow: ellipsis;
  }

  .mode {
    color: var(--text-muted);
    font-size: var(--fs-md);
  }

  .tag,
  .chip {
    flex: none;
    padding: 0 var(--space-2);
    border-radius: var(--radius-sm);
    font-size: var(--fs-xs);
    font-weight: 600;
    line-height: 1.6;
  }

  .tag {
    background: var(--surface-hover);
    color: var(--text);
  }

  .chip {
    border: 1px solid var(--border-strong);
    color: var(--text-muted);
  }

  .status {
    display: inline-flex;
    flex: none;
    align-items: center;
    gap: var(--space-2);
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
    white-space: nowrap;
  }

  .dot {
    width: var(--space-2);
    height: var(--space-2);
    border-radius: 50%;
    background: var(--text-faint);
  }

  .busy .dot {
    background: var(--info);
    animation: pulse calc(var(--duration-base) * 4) var(--ease-out) infinite alternate;
  }

  .live .dot {
    background: var(--success);
  }

  .live {
    color: var(--text);
  }

  .attention .dot {
    background: var(--warning);
  }

  @keyframes pulse {
    from {
      opacity: 1;
    }

    to {
      opacity: 0.35;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .busy .dot {
      animation: none;
    }
  }

  .actions {
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-1);
  }

  .refresh-error {
    color: var(--danger);
    font-size: var(--fs-sm);
  }
</style>
