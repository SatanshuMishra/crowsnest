<script lang="ts">
  import { onMount, untrack } from "svelte";
  import Onboarding from "./components/Onboarding.svelte";
  import Toast from "./components/Toast.svelte";
  import TopBar, { type Screen } from "./components/TopBar.svelte";
  import {
    INTRO_VERSION,
    introAudience,
    introCanOpen,
    introInterrupted,
    setupToast,
    type SetupOutcome,
  } from "./lib/intro";
  import { app } from "./lib/state.svelte";
  import { toasts } from "./lib/toast.svelte";
  import type { Settings as SettingsValues } from "./lib/types";
  import Monitor from "./views/Monitor.svelte";
  import Settings from "./views/Settings.svelte";
  import Setup from "./views/Setup.svelte";

  const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");
  app.systemDark = darkQuery.matches;
  let startError = $state<string | null>(null);
  let introOffered = false;
  let introBefore = $state.raw<SettingsValues | null>(null);
  let onboardingClosing = $state(false);

  onMount(() => {
    const onSchemeChange = (event: MediaQueryListEvent) => {
      app.systemDark = event.matches;
    };
    darkQuery.addEventListener("change", onSchemeChange);
    let unlisten: (() => void) | undefined;
    app
      .start()
      .then((stop) => {
        unlisten = stop;
      })
      .catch((error: unknown) => {
        startError = String(error);
      });
    return () => {
      darkQuery.removeEventListener("change", onSchemeChange);
      unlisten?.();
    };
  });

  $effect(() => {
    const root = document.documentElement;
    root.dataset.theme = app.theme;
    root.dataset.palette = app.settings?.palette ?? "muted";
    root.dataset.density = app.settings?.density ?? "comfortable";
    root.dataset.textSize = app.settings?.textSize ?? "normal";
  });

  const needsSetup = $derived(
    app.snapshot?.phase === "needsInstall" || app.snapshot?.phase === "needsApplicationId",
  );
  const screen = $derived(app.view === "settings" ? "settings" : needsSetup ? "setup" : "monitor");
  const ready = $derived(startError === null && app.snapshot !== null && app.settings !== null);
  const barScreen = $derived<Screen>(ready ? screen : "boot");

  $effect(() => {
    const settings = app.settings;
    const phase = app.snapshot?.phase;
    if (introOffered || !ready || !settings || !phase || app.setup !== null) {
      return;
    }
    const audience = introAudience(settings.introSeen);
    if (audience === null) {
      introOffered = true;
    } else if (introCanOpen(phase, screen === "monitor")) {
      introOffered = true;
      untrack(() => (introBefore ??= $state.snapshot(settings)));
      app.setup = audience;
    }
  });

  $effect(() => {
    if (!onboardingClosing && introInterrupted(app.setup, app.snapshot?.phase)) {
      app.setup = null;
      introOffered = false;
    }
  });

  function closeOnboarding(outcome: SetupOutcome) {
    const audience = app.setup;
    app.setup = null;
    onboardingClosing = false;
    if (audience === null) {
      return;
    }
    if (audience !== "tour") {
      introBefore = null;
      void app.update({ introSeen: INTRO_VERSION });
    }
    const toast = setupToast(audience, outcome);
    if (toast) {
      toasts.show(toast);
    }
  }
</script>

<div class="window">
  <TopBar snapshot={app.snapshot} screen={barScreen} />
  <div class="body">
    {#if startError}
      <main class="boot" role="alert">
        <h1>Crowsnest could not start</h1>
        <p>{startError}</p>
      </main>
    {:else if !app.snapshot || !app.settings}
      <main class="boot" aria-busy="true">
        <span class="visually-hidden">Starting Crowsnest</span>
      </main>
    {:else}
      {#key screen}
        <div class="screen" inert={app.setup !== null}>
          {#if screen === "settings"}
            <Settings />
          {:else if screen === "setup"}
            <Setup />
          {:else}
            <Monitor />
          {/if}
        </div>
      {/key}
      {#if app.setup}
        <Onboarding
          audience={app.setup}
          settings={app.settings}
          before={app.setup === "tour" ? null : introBefore}
          onclosing={() => (onboardingClosing = true)}
          onclose={closeOnboarding}
        />
      {/if}
    {/if}
  </div>
  <Toast />
</div>

<style>
  .window {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .body {
    position: relative;
    flex: 1;
    min-height: 0;
  }

  .screen {
    height: 100%;
    animation: screen-in var(--duration-base) var(--ease-out);
  }

  @keyframes screen-in {
    from {
      opacity: 0;
      translate: 0 var(--space-2);
    }
  }

  .boot {
    display: grid;
    place-content: center;
    gap: var(--space-2);
    height: 100%;
    padding: var(--space-6);
    text-align: center;
  }

  .boot h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--fs-xl);
    font-weight: var(--weight-strong);
  }

  .boot p {
    margin: 0;
    color: var(--text-muted);
  }
</style>
