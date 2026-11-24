<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import logo from "../../assets/logo.svg";
  import ApplicationIdField from "../components/settings/ApplicationIdField.svelte";
  import InstallPicker from "../components/settings/InstallPicker.svelte";
  import Button from "../components/ui/Button.svelte";
  import Icon from "../components/ui/Icon.svelte";
  import { app } from "../lib/state.svelte";

  const DEVELOPERS_URL = "https://developers.wargaming.net/applications/";

  const id = $props.id();
  let linkError = $state<string | null>(null);

  const needsApplicationId = $derived(app.snapshot?.phase === "needsApplicationId");

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

<main class="setup">
  <div class="column">
    <header class="brand">
      <img class="mark" src={logo} alt="" />
      <h1>Crowsnest</h1>
    </header>

    <section class="step" aria-labelledby="{id}-step">
      {#if needsApplicationId}
        <h2 id="{id}-step">Add a Wargaming application ID</h2>
        <p>
          Crowsnest reads public player statistics from the Wargaming API. The API needs a free application ID,
          which you can create as a Standalone application on
          <a href={DEVELOPERS_URL} onclick={openDevelopers}>developers.wargaming.net</a>.
        </p>
        <div class="feedback" aria-live="polite">
          {#if linkError}
            <p class="error"><Icon name="error" /><span>{linkError}</span></p>
          {/if}
        </div>
        <ApplicationIdField />
      {:else}
        <h2 id="{id}-step">Find World of Warships</h2>
        <p>
          Crowsnest could not find the game in the usual places: Wargaming Game Center, the Windows registry, Steam
          libraries and C:\Games. Detect it again, or choose its folder yourself.
        </p>
        <InstallPicker />
      {/if}
    </section>

    <footer class="footer">
      <Button onclick={() => (app.view = "settings")}>
        <Icon name="settings" />
        Open settings
      </Button>
    </footer>
  </div>
</main>

<style>
  .setup {
    display: grid;
    justify-items: center;
    align-items: safe center;
    height: 100%;
    overflow-y: auto;
    padding: var(--space-6) var(--space-4);
  }

  .column {
    display: grid;
    gap: var(--space-5);
    width: 100%;
    max-width: calc(var(--space-6) * 18);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .mark {
    width: calc(var(--space-6) + var(--space-3));
    height: calc(var(--space-6) + var(--space-3));
  }

  h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--fs-xl);
    font-weight: 600;
  }

  .step {
    display: grid;
    gap: var(--space-3);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--fs-lg);
    font-weight: 600;
  }

  p {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-md);
  }

  a {
    color: var(--accent);
    font-weight: 600;
    text-decoration: underline;
  }

  a:hover {
    color: var(--accent-strong);
  }

  .feedback:empty {
    display: none;
  }

  .error {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    color: var(--danger);
    font-size: var(--fs-sm);
  }

  .footer {
    display: flex;
    padding-top: var(--space-4);
    border-top: 1px solid var(--border);
  }
</style>
