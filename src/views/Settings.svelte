<script lang="ts" module>
  import type { SectionKey } from "../lib/settingsIndex";

  let lastSection: SectionKey = "game";
</script>

<script lang="ts">
  import { tick } from "svelte";
  import About from "../components/settings/About.svelte";
  import AppearancePanel from "../components/settings/AppearancePanel.svelte";
  import GamePanel from "../components/settings/GamePanel.svelte";
  import StatisticsPanel from "../components/settings/StatisticsPanel.svelte";
  import UpdatesPanel from "../components/settings/UpdatesPanel.svelte";
  import WindowPanel from "../components/settings/WindowPanel.svelte";
  import Icon from "../components/ui/Icon.svelte";
  import IconButton from "../components/ui/IconButton.svelte";
  import { focusViewHeading } from "../lib/focus";
  import { preview } from "../lib/preview.svelte";
  import { SECTIONS, resultsTitle, searchSettings, sectionTitle, type SettingEntry } from "../lib/settingsIndex";
  import { app } from "../lib/state.svelte";

  const paneId = $props.id();

  let section = $state<SectionKey>(lastSection);
  let query = $state("");
  let found = $state<string | null>(null);
  let pane = $state<HTMLDivElement>();

  const settings = $derived(app.settings);
  const results = $derived(searchSettings(query));
  const searching = $derived(query.trim() !== "");
  const current = $derived(SECTIONS.find((entry) => entry.key === section) ?? SECTIONS[0]!);

  function go(key: SectionKey) {
    preview.clear();
    section = key;
    lastSection = key;
    query = "";
    found = null;
    if (pane) {
      pane.scrollTop = 0;
    }
  }

  async function reveal(entry: SettingEntry) {
    go(entry.section);
    found = entry.key;
    await tick();
    const target = pane?.querySelector<HTMLElement>(`[data-setting="${entry.key}"]`);
    target?.scrollIntoView({ block: "center" });
    target?.querySelector<HTMLElement>("button:not([tabindex='-1']), input, a[href], summary")?.focus();
  }

  function onSearchKey(event: KeyboardEvent) {
    if (event.key === "Escape" && query !== "") {
      event.preventDefault();
      query = "";
    }
  }
</script>

<main class="settings">
  <nav class="nav" aria-label="Settings sections">
    <header class="header">
      <IconButton icon="back" label="Back" onclick={() => (app.view = "monitor")} />
      <h1 tabindex="-1" {@attach focusViewHeading}>Settings</h1>
    </header>
    <label class="search">
      <Icon name="search" size={16} />
      <input
        type="search"
        placeholder="Search settings"
        aria-label="Search settings"
        aria-controls={paneId}
        bind:value={query}
        onkeydown={onSearchKey}
      />
    </label>
    <ul class="sections">
      {#each SECTIONS as entry (entry.key)}
        {@const selected = !searching && entry.key === section}
        <li>
          <button
            type="button"
            class="section-link"
            class:selected
            aria-current={selected ? "true" : undefined}
            onclick={() => go(entry.key)}
          >
            <span class="name">{entry.title}</span>
            <span class="summary">{entry.summary}</span>
          </button>
        </li>
      {/each}
    </ul>
  </nav>

  <div class="pane" id={paneId} data-scroll-root bind:this={pane}>
    <div class="column">
      {#if app.saveError}
        <p class="save-error" role="alert">
          <Icon name="error" />
          <span>Your last change was not saved: {app.saveError}</span>
        </p>
      {/if}

      <p class="visually-hidden" aria-live="polite">{searching ? resultsTitle(query, results.length) : ""}</p>

      {#if searching}
        <section class="panel" aria-labelledby="{paneId}-results">
          <h2 id="{paneId}-results">{resultsTitle(query, results.length)}</h2>
          {#if results.length > 0}
            <ul class="results">
              {#each results as entry (entry.key)}
                <li>
                  <button type="button" class="result" onclick={() => reveal(entry)}>
                    <span class="result-head">
                      <span class="result-label">{entry.label}</span>
                      <span class="result-section">{sectionTitle(entry.section)}</span>
                    </span>
                    <span class="result-description">{entry.description}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {:else}
            <p class="description">Try another word, such as colour, tray or server.</p>
          {/if}
        </section>
      {:else if settings}
        {#key section}
          <section class="panel" aria-labelledby="{paneId}-title">
            <header class="panel-header">
              <h2 id="{paneId}-title">{current.title}</h2>
              {#if current.description}
                <p class="description">{current.description}</p>
              {/if}
            </header>
            <div class="panel-body">
              {#if section === "game"}
                <GamePanel {settings} {found} />
              {:else if section === "statistics"}
                <StatisticsPanel {settings} {found} />
              {:else if section === "appearance"}
                <AppearancePanel {settings} {found} />
              {:else if section === "window"}
                <WindowPanel {settings} {found} />
              {:else if section === "updates"}
                <UpdatesPanel {settings} {found} />
              {:else}
                <div data-setting="logs" data-found={found === "logs" || undefined}>
                  <About />
                </div>
              {/if}
            </div>
          </section>
        {/key}
      {/if}
    </div>
  </div>
</main>

<style>
  .settings {
    display: flex;
    height: 100%;
  }

  .nav {
    display: flex;
    flex: none;
    flex-direction: column;
    gap: var(--space-4);
    width: var(--nav-width);
    padding: var(--space-5) var(--space-3) var(--space-5) var(--space-5);
    overflow-y: auto;
    border-right: 1px solid var(--border);
  }

  .header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-left: calc(-1 * var(--space-2));
  }

  h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--fs-xl);
    font-weight: var(--weight-strong);
    letter-spacing: var(--tracking-display);
  }

  .search {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: 32px;
    padding: 0 var(--space-2);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
  }

  .search:focus-within {
    outline: 2px solid var(--focus);
    outline-offset: 2px;
  }

  .search input {
    flex: 1;
    min-width: 0;
    min-height: 30px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-md);
    outline: none;
  }

  .search input::placeholder {
    color: var(--text-faint);
  }

  .sections,
  .results {
    display: grid;
    gap: 2px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .section-link,
  .result {
    display: grid;
    gap: 2px;
    width: 100%;
    min-height: 40px;
    padding: var(--space-2) var(--space-3);
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .section-link {
    margin-left: calc(-1 * var(--space-3));
    width: calc(100% + var(--space-3));
    color: var(--text-muted);
  }

  .section-link:hover,
  .result:hover,
  .selected {
    background: var(--surface-hover);
    color: var(--text);
  }

  .name {
    font-size: var(--fs-md);
  }

  .selected .name {
    font-weight: var(--weight-strong);
  }

  .summary {
    color: var(--text-muted);
    font-size: var(--fs-xs);
  }

  .pane {
    flex: 1;
    min-width: 0;
    padding: var(--space-5) var(--space-6) var(--space-6);
    overflow-y: auto;
  }

  .column {
    display: grid;
    gap: var(--space-4);
    max-width: var(--settings-width);
  }

  .panel {
    display: grid;
    gap: var(--space-4);
  }

  .panel-header {
    display: grid;
    gap: var(--space-1);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--border);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--fs-xl);
    font-weight: var(--weight-strong);
    letter-spacing: var(--tracking-display);
  }

  .description {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .panel-body {
    display: grid;
    gap: var(--space-3);
  }

  .save-error {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    margin: 0;
    color: var(--danger);
    font-size: var(--fs-md);
  }

  .result-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .result-label {
    font-size: var(--fs-md);
    font-weight: var(--weight-medium);
  }

  .result-section,
  .result-description {
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }
</style>
