<script lang="ts">
  import { onMount } from "svelte";
  import { app } from "./lib/state.svelte";
  import Monitor from "./views/Monitor.svelte";
  import Settings from "./views/Settings.svelte";
  import Setup from "./views/Setup.svelte";

  const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");
  let systemDark = $state(darkQuery.matches);
  let startError = $state<string | null>(null);

  onMount(() => {
    const onSchemeChange = (event: MediaQueryListEvent) => {
      systemDark = event.matches;
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

  const theme = $derived.by(() => {
    const chosen = app.settings?.theme ?? "system";
    if (chosen === "system") {
      return systemDark ? "dark" : "light";
    }
    return chosen;
  });

  $effect(() => {
    const root = document.documentElement;
    root.dataset.theme = theme;
    root.dataset.density = app.settings?.density ?? "comfortable";
    root.dataset.textSize = app.settings?.textSize ?? "normal";
  });

  const needsSetup = $derived(
    app.snapshot?.phase === "needsInstall" || app.snapshot?.phase === "needsApplicationId",
  );
</script>

{#if startError}
  <main class="boot" role="alert">
    <h1>Crowsnest could not start</h1>
    <p>{startError}</p>
  </main>
{:else if !app.snapshot || !app.settings}
  <main class="boot" aria-busy="true">
    <span class="visually-hidden">Starting Crowsnest</span>
  </main>
{:else if app.view === "settings"}
  <Settings />
{:else if needsSetup}
  <Setup />
{:else}
  <Monitor />
{/if}

<style>
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
  }

  .boot p {
    margin: 0;
    color: var(--text-muted);
  }
</style>
