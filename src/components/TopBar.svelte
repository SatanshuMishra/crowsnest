<script lang="ts" module>
  export type Screen = "monitor" | "settings" | "setup" | "boot";
</script>

<script lang="ts">
  import PinButton from "./PinButton.svelte";
  import UpdateNotice from "./UpdateNotice.svelte";
  import WindowControls from "./WindowControls.svelte";
  import IconButton from "./ui/IconButton.svelte";
  import { backend } from "../lib/backend";
  import { connectionStatus } from "../lib/connection";
  import { focusViewHeading } from "../lib/focus";
  import { matchType, regionLabel, regionName } from "../lib/format";
  import {
    REFRESH_MIN_SPIN_MS,
    canRefresh,
    cooldownSeconds,
    followRefresh,
    refreshLabel,
    refreshToast,
    type RefreshWatch,
  } from "../lib/refresh";
  import { app } from "../lib/state.svelte";
  import { toasts } from "../lib/toast.svelte";
  import type { Snapshot } from "../lib/types";
  import { windowDrag } from "../lib/window";

  interface Props {
    snapshot: Snapshot | null;
    screen: Screen;
  }

  let { snapshot, screen }: Props = $props();

  const CLOCK_TICK_MS = 250;

  let refreshing = $state(false);
  let refreshWatch: RefreshWatch | null = null;
  let lastRefreshAt = $state<number | null>(null);
  let now = $state(performance.now());

  const battle = $derived(snapshot?.battle ?? null);
  const region = $derived(battle?.region ?? snapshot?.install?.region ?? null);
  const onMonitor = $derived(screen === "monitor");
  const titled = $derived(onMonitor || screen === "settings");
  const lastBattle = $derived(onMonitor && battle !== null && snapshot?.phase === "waiting");
  const status = $derived(snapshot ? connectionStatus(snapshot) : null);
  const watchedFolder = $derived(app.settings?.replaysDirOverride ?? snapshot?.install?.replaysDir ?? null);
  const statusTitle = $derived(
    status?.connected && watchedFolder ? `Watching ${watchedFolder} for new battles` : undefined,
  );
  const availability = $derived({
    hasBattle: battle !== null,
    busy: refreshing || snapshot?.phase === "loading",
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

  $effect(() => {
    const current = snapshot;
    if (refreshWatch === null || !current) {
      return;
    }
    const progress = followRefresh(refreshWatch, current);
    if (progress.kind === "watching") {
      refreshWatch = progress.watch;
      return;
    }
    refreshWatch = null;
    if (progress.kind === "finished" && current.battle) {
      const toast = refreshToast(current.notice, [current.battle.allies, current.battle.enemies], () => void refresh());
      if (toast.tone === "error") {
        lastRefreshAt = null;
      }
      toasts.show(toast);
    }
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
    refreshWatch = battle ? { battleId: battle.id, started: false } : null;
    lastRefreshAt = startedAt;
    now = startedAt;
    try {
      await backend.refresh();
    } catch (error) {
      refreshWatch = null;
      lastRefreshAt = null;
      toasts.show({
        tone: "error",
        title: "Refresh failed",
        body: String(error),
        action: { label: "Try again", run: () => void refresh() },
      });
    } finally {
      await pause(REFRESH_MIN_SPIN_MS - (performance.now() - startedAt));
      refreshing = false;
    }
  }

  function toggleSettings() {
    app.view = app.view === "settings" ? "monitor" : "settings";
  }
</script>

<header class="top-bar" use:windowDrag>
  <div class="match">
    {#if titled}
      <svelte:element
        this={onMonitor ? "h1" : "p"}
        class="title"
        tabindex={onMonitor ? -1 : undefined}
        {@attach onMonitor && focusViewHeading}
      >
        {#if onMonitor && battle}
          <span class="visually-hidden">Crowsnest, </span>{matchType(battle.arena.gameType, battle.arena.matchGroup)}
        {:else}
          Crowsnest
        {/if}
      </svelte:element>
      {#if lastBattle}
        <span class="tag">Last battle</span>
      {/if}
      {#if region}
        <span class="chip" title="Game server: {regionName(region)}">{regionLabel(region)} server</span>
      {/if}
    {/if}
  </div>

  {#if status}
    <p
      class="status"
      class:connected={status.connected}
      data-activity={status.activity}
      title={statusTitle}
      aria-live="polite"
      aria-atomic="true"
    >
      <svg class="radar" viewBox="0 0 16 16" width="14" height="14" aria-hidden="true" focusable="false">
        <circle class="ring" cx="8" cy="8" r="6.5" />
        <path class="sweep" d="M8 8V1.5a6.5 6.5 0 0 1 5.63 3.25Z" />
        <circle class="core" cx="8" cy="8" r="2.25" />
      </svg>
      <span class="label">{status.label}</span>
      <span class="detail">{status.detail}</span>
    </p>
  {/if}

  {#if snapshot && titled}
    <div class="actions" inert={app.setup !== null}>
      <UpdateNotice />
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
      <IconButton icon="settings" label="Settings" pressed={screen === "settings"} onclick={toggleSettings} />
    </div>
  {/if}

  <PinButton />
  <WindowControls />
</header>

<style>
  .top-bar {
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-4);
    height: var(--titlebar-height);
    padding-left: var(--space-5);
    border-bottom: 1px solid var(--border);
    background: var(--bg);
    user-select: none;
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

  .title {
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
  .chip::before,
  .detail::before {
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
  }

  .label {
    color: var(--warning);
    font-weight: var(--weight-medium);
    transition: color var(--duration-base) var(--ease-out);
  }

  .connected .label {
    color: var(--text);
  }

  .radar {
    flex: none;
    color: var(--warning);
    overflow: visible;
    transition: color var(--duration-base) var(--ease-out);
  }

  .connected .radar {
    color: var(--success);
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

  [data-activity="busy"] .sweep {
    animation-duration: 1s;
  }

  [data-activity="steady"] .sweep {
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
</style>
