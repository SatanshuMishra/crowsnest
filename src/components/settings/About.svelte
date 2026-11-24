<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { backend } from "../../lib/backend";
  import { app } from "../../lib/state.svelte";
  import Button from "../ui/Button.svelte";
  import Icon from "../ui/Icon.svelte";

  interface Link {
    href: string;
    label: string;
    detail: string;
  }

  const links: Link[] = [
    {
      href: "https://github.com/SatanshuMishra/crowsnest",
      label: "Crowsnest on GitHub",
      detail: "Source code, releases and issues",
    },
    {
      href: "https://wows-numbers.com/",
      label: "WoWS Numbers",
      detail: "Expected values for personal rating, with thanks",
    },
    {
      href: "https://worldofwarships.com/",
      label: "World of Warships",
      detail: "The game, by Wargaming",
    },
  ];

  let linkError = $state<string | null>(null);
  let logError = $state<string | null>(null);
  let openingLogs = $state(false);

  const version = $derived(app.info?.version ?? null);

  async function openLink(event: MouseEvent, href: string) {
    event.preventDefault();
    linkError = null;
    try {
      await openUrl(href);
    } catch {
      linkError = `Your browser could not be opened. The address is ${href}`;
    }
  }

  async function openLogs() {
    openingLogs = true;
    logError = null;
    try {
      await backend.openLogFolder();
    } catch {
      logError = "The log folder could not be opened. Try again, or restart Crowsnest.";
    } finally {
      openingLogs = false;
    }
  }
</script>

<div class="about">
  <p class="name">
    <span class="product">Crowsnest</span>
    <span class="version num">{version ? `Version ${version}` : "Version not known"}</span>
  </p>
  <p class="privacy">
    No account and no telemetry: your settings and caches stay on this computer. Statistics come from the
    public Wargaming API and expected values from WoWS Numbers.
  </p>

  <ul class="links">
    {#each links as link (link.href)}
      <li>
        <a href={link.href} onclick={(event) => openLink(event, link.href)}>
          {link.label}
          <Icon name="external" />
          <span class="visually-hidden">(opens in your browser)</span>
        </a>
        <span class="detail">{link.detail}</span>
      </li>
    {/each}
  </ul>
  <div class="feedback" aria-live="polite">
    {#if linkError}
      <p class="error"><Icon name="error" /><span>{linkError}</span></p>
    {/if}
  </div>

  <div class="logs">
    <Button busy={openingLogs} onclick={openLogs}>
      <Icon name="folder" />
      Open log folder
    </Button>
    <span class="detail">Attach the latest log when you report a problem.</span>
  </div>
  <div class="feedback" aria-live="polite">
    {#if logError}
      <p class="error"><Icon name="error" /><span>{logError}</span></p>
    {/if}
  </div>

  <p class="legal">© Wargaming.net. All rights reserved. Crowsnest is not affiliated with Wargaming.</p>
</div>

<style>
  .about {
    display: grid;
    gap: var(--space-3);
  }

  p {
    margin: 0;
  }

  .name {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--space-2);
  }

  .product {
    font-family: var(--font-display);
    font-size: var(--fs-lg);
    font-weight: 600;
  }

  .version,
  .detail,
  .legal {
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .privacy {
    font-size: var(--fs-md);
  }

  .links {
    display: grid;
    gap: var(--space-1);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0 var(--space-3);
  }

  a {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    min-height: var(--target-min);
    color: var(--accent);
    font-size: var(--fs-md);
    font-weight: 600;
    text-decoration: underline;
    border-radius: var(--radius-sm);
  }

  a:hover {
    color: var(--accent-strong);
  }

  .logs {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-3);
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
</style>
