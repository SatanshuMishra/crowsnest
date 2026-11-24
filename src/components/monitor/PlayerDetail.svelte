<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Button from "../ui/Button.svelte";
  import Icon from "../ui/Icon.svelte";
  import IconButton from "../ui/IconButton.svelte";
  import StatCell from "./StatCell.svelte";
  import { formatWinRate, romanNumeral, shipClassBadge, type StatKind } from "../../lib/format";
  import { playerLinks } from "../../lib/links";
  import { lineValue } from "../../lib/team";
  import { statTier } from "../../lib/tiers";
  import type { ExpectedStats, Player, Region, Relation, Settings, StatLine } from "../../lib/types";

  interface Props {
    player: Player | null;
    region: Region | null;
    settings: Settings;
    onclose: () => void;
  }

  let { player, region, settings, onclose }: Props = $props();
  const id = $props.id();

  let dialog = $state<HTMLDialogElement>();
  let linkError = $state<string | null>(null);

  interface LineColumn {
    id: string;
    name: string;
    line: StatLine | null;
    expected: ExpectedStats | null;
  }

  const ROWS: ReadonlyArray<{ kind: StatKind; name: string }> = [
    { kind: "battles", name: "Battles" },
    { kind: "winRate", name: "Win rate" },
    { kind: "personalRating", name: "Personal rating" },
    { kind: "damage", name: "Average damage" },
    { kind: "frags", name: "Average frags" },
    { kind: "xp", name: "Average experience" },
  ];

  const RELATIONS: Record<Relation, string> = {
    me: "You",
    ally: "Ally",
    enemy: "Enemy",
  };

  const ACCOUNT_RATED: ReadonlySet<StatKind> = new Set<StatKind>(["winRate", "personalRating"]);
  const RATIO_RATED: ReadonlySet<StatKind> = new Set<StatKind>(["damage", "frags"]);

  const stats = $derived(player?.stats ?? null);
  const badge = $derived(player?.ship ? shipClassBadge(player.ship.class) : null);
  const links = $derived(player ? playerLinks(region, player.stats, player.name) : []);
  const lineColumns = $derived.by<LineColumn[]>(() => {
    if (!player || stats?.kind !== "loaded") {
      return [];
    }
    return [
      { id: "overall", name: "Overall", line: stats.overall, expected: null },
      { id: "overall-solo", name: "Solo", line: stats.overallSolo, expected: null },
      { id: "ship", name: "Ship", line: stats.ship, expected: player.expected },
      { id: "ship-solo", name: "Ship solo", line: stats.shipSolo, expected: player.expected },
    ];
  });

  $effect(() => {
    if (!dialog) {
      return;
    }
    if (player && !dialog.open) {
      linkError = null;
      dialog.showModal();
    } else if (!player && dialog.open) {
      dialog.close();
    }
  });

  function close() {
    dialog?.close();
  }

  function onBackdrop(event: MouseEvent) {
    if (event.target === dialog) {
      close();
    }
  }

  async function follow(url: string) {
    linkError = null;
    try {
      await openUrl(url);
    } catch (error) {
      linkError = `The link could not be opened: ${String(error)}`;
    }
  }

  function rated(column: LineColumn, kind: StatKind): boolean {
    return ACCOUNT_RATED.has(kind) || (column.expected !== null && RATIO_RATED.has(kind));
  }
</script>

<dialog bind:this={dialog} class="sheet" aria-labelledby="{id}-title" onclose={onclose} onclick={onBackdrop}>
  {#if player && stats}
    <div class="body">
      <header>
        <div class="heading">
          <p class="relation {player.relation}">{RELATIONS[player.relation]}</p>
          <h2 id="{id}-title">
            {#if player.clanTag}<span class="clan">[{player.clanTag}]</span>{/if}
            {player.name}
          </h2>
          {#if player.ship}
            <p class="ship">
              <span class="tier"><span class="visually-hidden">Tier </span>{romanNumeral(player.ship.tier)}</span>
              {player.ship.name}{#if badge}<span class="class-name"> · {badge.name}</span>{/if}
            </p>
          {:else}
            <p class="ship" title="Ship id {player.shipId}">Unknown ship</p>
          {/if}
        </div>
        <IconButton icon="close" label="Close player details" onclick={close} />
      </header>

      {#if stats.kind === "loaded"}
        <table class="lines">
          <caption class="visually-hidden">Random battle statistics</caption>
          <thead>
            <tr>
              <td></td>
              {#each lineColumns as column (column.id)}
                <th scope="col" class="num">{column.name}</th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each ROWS as row (row.kind)}
              <tr>
                <th scope="row">{row.name}</th>
                {#each lineColumns as column (column.id)}
                  {@const value = column.line ? lineValue(column.line, row.kind) : null}
                  <StatCell
                    kind={row.kind}
                    label="{row.name}, {column.name.toLowerCase()}"
                    {value}
                    tier={statTier(row.kind, value, column.expected)}
                    rated={rated(column, row.kind)}
                    colored={settings.coloredValues}
                  />
                {/each}
              </tr>
            {/each}
            <tr>
              <th scope="row">Survival rate</th>
              {#each lineColumns as column (column.id)}
                <td class="num plain">{formatWinRate(column.line?.survivalRate)}</td>
              {/each}
            </tr>
          </tbody>
        </table>
        <p class="note">
          Ship columns cover this ship only; solo columns leave out battles played in a division. Personal rating uses
          expected values from wows-numbers.com.
        </p>
      {:else if stats.kind === "pending"}
        <p class="state" aria-busy="true">Loading statistics from Wargaming.</p>
      {:else if stats.kind === "bot"}
        <p class="state">This is a bot. Bots have no statistics.</p>
      {:else if stats.kind === "hidden"}
        <p class="state with-icon">
          <Icon name="eyeOff" />
          This player has hidden their profile, so Wargaming shares no statistics for it.
        </p>
      {:else if stats.kind === "notFound"}
        <p class="state">Wargaming has no account with this name on this server.</p>
      {:else if stats.kind === "failed"}
        <p class="state with-icon error">
          <Icon name="error" />
          <span>Statistics are unavailable: {stats.reason}</span>
        </p>
      {/if}

      {#if links.length > 0}
        <section class="links" aria-labelledby="{id}-links">
          <h3 id="{id}-links">Profiles</h3>
          <div class="link-row">
            {#each links as link (link.id)}
              <Button size="sm" onclick={() => follow(link.url)}>
                {link.label}
                <Icon name="external" size={14} />
                <span class="visually-hidden">(opens in your browser)</span>
              </Button>
            {/each}
          </div>
          {#if linkError}
            <p class="link-error" role="alert">{linkError}</p>
          {/if}
        </section>
      {:else if (stats.kind === "loaded" || stats.kind === "hidden") && region === null}
        <p class="note">Profile links need the server region, which could not be detected.</p>
      {/if}
    </div>
  {/if}
</dialog>

<style>
  .sheet {
    position: fixed;
    inset: 0 0 0 auto;
    width: min(100vw, calc(var(--space-6) * 13));
    max-width: 100vw;
    height: 100%;
    max-height: 100%;
    margin: 0;
    padding: 0;
    border: 0;
    border-left: 1px solid var(--border-strong);
    background: var(--surface-raised);
    color: var(--text);
    box-shadow: var(--shadow-pop);
  }

  .sheet[open] {
    animation: slide-in var(--duration-base) var(--ease-out);
  }

  .sheet::backdrop {
    background: color-mix(in srgb, var(--bg) 50%, transparent);
  }

  @keyframes slide-in {
    from {
      transform: translateX(var(--space-5));
      opacity: 0;
    }

    to {
      transform: none;
      opacity: 1;
    }
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    min-height: 100%;
    padding: var(--space-4);
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .heading {
    min-width: 0;
  }

  .relation {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-xs);
    font-weight: 600;
    text-transform: uppercase;
  }

  .relation.ally {
    color: var(--ally);
  }

  .relation.enemy {
    color: var(--enemy);
  }

  h2 {
    margin: var(--space-1) 0 0;
    font-family: var(--font-display);
    font-size: var(--fs-xl);
    font-weight: 600;
    overflow-wrap: anywhere;
  }

  .clan {
    color: var(--text-muted);
  }

  .ship {
    margin: var(--space-1) 0 0;
    color: var(--text-muted);
    font-size: var(--fs-md);
  }

  .tier {
    font-weight: 600;
  }

  .lines {
    width: 100%;
    border-collapse: collapse;
  }

  .lines th {
    padding: var(--space-1) var(--space-2);
    border-bottom: 1px solid var(--border);
    color: var(--text-muted);
    font-size: var(--fs-sm);
    font-weight: 600;
    text-align: left;
    white-space: nowrap;
  }

  .lines thead th {
    font-size: var(--fs-xs);
    text-align: right;
  }

  .lines tr {
    height: var(--row-height);
  }

  .lines :global(td) {
    font-size: var(--fs-md);
  }

  .plain {
    padding: 0 var(--space-2);
    border-bottom: 1px solid var(--border);
    text-align: right;
    white-space: nowrap;
  }

  .note {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .state {
    margin: 0;
    padding: var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-muted);
    font-size: var(--fs-md);
  }

  .with-icon {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
  }

  .error {
    color: var(--text);
    border-color: var(--danger);
  }

  .error :global(svg) {
    color: var(--danger);
  }

  .links {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin-top: auto;
  }

  h3 {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-xs);
    font-weight: 600;
    text-transform: uppercase;
  }

  .link-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .link-error {
    margin: 0;
    color: var(--danger);
    font-size: var(--fs-sm);
  }
</style>
