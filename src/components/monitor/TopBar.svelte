<script lang="ts">
  import UpdateNotice from "../UpdateNotice.svelte";
  import IconButton from "../ui/IconButton.svelte";
  import { backend } from "../../lib/backend";
  import { matchType, regionLabel, regionName } from "../../lib/format";
  import { REFRESH_MIN_SPIN_MS, canRefresh, cooldownSeconds, refreshLabel } from "../../lib/refresh";
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

  const CLOCK_TICK_MS = 250;

  let refreshing = $state(false);
  let refreshError = $state<string | null>(null);
  let lastRefreshAt = $state<number | null>(null);
  let now = $state(performance.now());

  const battle = $derived(snapshot.battle);
  const region = $derived(battle?.region ?? snapshot.install?.region ?? null);
  const lastBattle = $derived(battle !== null && snapshot.phase === "waiting");
  const status = $derived(STATUS[snapshot.phase]);
  const availability = $derived({
    hasBattle: battle !== null,
    busy: refreshing || snapshot.phase === "loading",
    cooldown: cooldownSeconds(lastRefreshAt, now),
  });

  $effect(() => {
    const since = lastRefreshAt;
    if (since === null) {
      return;
    }
    const timer = setInterval(() => {
      now = performance.now();
      if (cooldownSeconds(since, now) === 0) {
        clearInterval(timer);
      }
    }, CLOCK_TICK_MS);
    return () => clearInterval(timer);
  });

  function pause(milliseconds: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, Math.max(milliseconds, 0)));
  }

  async function refresh() {
    if (!canRefresh(availability)) {
      return;
    }
    const startedAt = performance.now();
    refreshing = true;
    refreshError = null;
    lastRefreshAt = startedAt;
    now = startedAt;
    try {
      await backend.refresh();
    } catch (error) {
      refreshError = String(error);
      lastRefreshAt = null;
    } finally {
      await pause(REFRESH_MIN_SPIN_MS - (performance.now() - startedAt));
      refreshing = false;
    }
  }

  function openSettings() {
    app.view = "settings";
  }
</script>

<header class="top-bar">
  <div class="match">
    {#if battle}
      <h1>
        <span class="visually-hidden">Crowsnest, </span>{matchType(battle.arena.gameType, battle.arena.matchGroup)}
      </h1>
      {#if lastBattle}
        <span class="tag">Last battle</span>
      {/if}
    {:else}
      <h1>Crowsnest</h1>
    {/if}
    {#if region}
      <span class="chip" title="Game server: {regionName(region)}">{regionLabel(region)} server</span>
    {/if}
  </div>

  <p class="status {status.tone}" aria-live="polite" aria-atomic="true">
    <svg class="radar" viewBox="0 0 16 16" width="14" height="14" aria-hidden="true" focusable="false">
      <circle class="ring" cx="8" cy="8" r="6.5" />
      <path class="sweep" d="M8 8V1.5a6.5 6.5 0 0 1 5.63 3.25Z" />
      <circle class="core" cx="8" cy="8" r="2.25" />
    </svg>
    {status.label}
  </p>

  <div class="actions">
    <UpdateNotice />
    {#if refreshError}
      <span class="refresh-error" role="alert" title={refreshError}>Refresh failed</span>
    {/if}
    {#if availability.cooldown > 0 && !availability.busy}
      <span class="cooldown num" aria-hidden="true">{availability.cooldown}s</span>
    {/if}
    <IconButton
      icon="refresh"
      label={refreshLabel(availability)}
      busy={availability.hasBattle && availability.busy}
      disabled={!availability.hasBattle}
      aria-disabled={availability.hasBattle && !canRefresh(availability) ? "true" : undefined}
      onclick={refresh}
    />
    <IconButton icon="settings" label="Settings" onclick={openSettings} />
  </div>
</header>

<style>
  .top-bar {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    min-height: calc(var(--target-min) + var(--space-5));
    padding: 0 var(--space-3) 0 var(--space-5);
    border-bottom: 1px solid var(--border);
    background: var(--bg);
  }

  .match {
    display: flex;
    flex: 1;
    align-items: baseline;
    gap: var(--space-2);
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
  }

  h1 {
    margin: 0;
    overflow: hidden;
    font-family: var(--font-display);
    font-size: var(--fs-lg);
    font-weight: var(--weight-strong);
    text-overflow: ellipsis;
  }

  .tag,
  .chip {
    flex: none;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .tag::before,
  .chip::before {
    content: "·" / "";
    margin-right: var(--space-2);
    color: var(--text-faint);
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
    transition: color var(--duration-base) var(--ease-out);
  }

  .radar {
    flex: none;
    color: var(--text-faint);
    overflow: visible;
    transition: color var(--duration-base) var(--ease-out);
  }

  .ring {
    fill: none;
    stroke: currentColor;
    stroke-width: 1.25;
    opacity: 0.6;
  }

  .sweep {
    fill: currentColor;
    opacity: 0.5;
    transform-origin: 8px 8px;
    animation: sweep 3s linear infinite;
  }

  .core {
    fill: currentColor;
  }

  .busy .radar {
    color: var(--info);
  }

  .busy .sweep {
    animation-duration: 1s;
  }

  .live {
    color: var(--text);
  }

  .live .radar {
    color: var(--success);
  }

  .attention,
  .attention .radar {
    color: var(--warning);
  }

  .live .sweep,
  .attention .sweep {
    opacity: 0;
    animation: none;
  }

  @keyframes sweep {
    to {
      rotate: 1turn;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .sweep {
      animation: none;
    }
  }

  .cooldown {
    color: var(--text-faint);
    font-size: var(--fs-xs);
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
