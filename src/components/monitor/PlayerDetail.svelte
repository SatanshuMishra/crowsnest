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

  let { player: requested, region, settings, onclose }: Props = $props();
  const id = $props.id();

  let player = $state.raw<Player | null>(null);

  $effect.pre(() => {
    if (requested) {
      player = requested;
    }
  });

  let dialog = $state<HTMLDialogElement>();
  let linkError = $state<string | null>(null);

  type Split = "all" | "solo" | "div2" | "div3";

  interface LineColumn {
    id: string;
    name: string;
    title: string;
    spoken: string;
    line: StatLine | null;
    expected: ExpectedStats | null;
  }

  interface LineGroup {
    id: string;
    name: string;
    pending: boolean;
    columns: LineColumn[];
  }

  const SPLITS: ReadonlyArray<{ id: Split; name: string; title: string; spoken: string }> = [
    { id: "all", name: "All", title: "All random battles", spoken: "all battles" },
    { id: "solo", name: "Solo", title: "Random battles played without a division", spoken: "solo" },
    { id: "div2", name: "Div 2", title: "Random battles in a two-player division", spoken: "two-player division" },
    { id: "div3", name: "Div 3", title: "Random battles in a three-player division", spoken: "three-player division" },
  ];

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
  const groups = $derived.by<LineGroup[]>(() => {
    if (!player || stats?.kind !== "loaded") {
      return [];
    }
    const account: LineGroup = {
      id: "account",
      name: "Account",
      pending: false,
      columns: columnsFor(
        "account",
        { all: stats.overall, solo: stats.overallSolo, div2: stats.overallDiv2, div3: stats.overallDiv3 },
        null,
      ),
    };
    const ship: LineGroup = {
      id: "ship",
      name: "This ship",
      pending: stats.shipState === "pending",
      columns: columnsFor(
        "ship",
        { all: stats.ship, solo: stats.shipSolo, div2: stats.shipDiv2, div3: stats.shipDiv3 },
        player.expected,
      ),
    };
    return stats.ship !== null || ship.pending ? [account, ship] : [account];
  });

  function columnsFor(
    group: string,
    lines: Record<Split, StatLine | null>,
    expected: ExpectedStats | null,
  ): LineColumn[] {
    return SPLITS.map((split) => ({
      id: `${group}-${split.id}`,
      name: split.name,
      title: split.title,
      spoken: split.spoken,
      line: lines[split.id],
      expected,
    }));
  }

  $effect(() => {
    if (!dialog) {
      return;
    }
    if (requested && !dialog.open) {
      linkError = null;
      dialog.showModal();
    } else if (!requested && dialog.open) {
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
        {#each groups as group (group.id)}
          <table class="lines">
            <caption>{group.name}</caption>
            <thead>
              <tr>
                <td></td>
                {#each group.columns as column (column.id)}
                  <th scope="col" class="num" title={column.title}>{column.name}</th>
                {/each}
              </tr>
            </thead>
            <tbody>
              {#each ROWS as row (row.kind)}
                <tr>
                  <th scope="row">{row.name}</th>
                  {#each group.columns as column (column.id)}
                    {@const value = column.line ? lineValue(column.line, row.kind) : null}
                    <StatCell
                      kind={row.kind}
                      label="{row.name}, {group.name.toLowerCase()}, {column.spoken}"
                      {value}
                      tier={statTier(row.kind, value, column.expected)}
                      rated={rated(column, row.kind)}
                      colored={settings.coloredValues}
                      pending={group.pending}
                    />
                  {/each}
                </tr>
              {/each}
              <tr>
                <th scope="row">Survival rate</th>
                {#each group.columns as column (column.id)}
                  <td class="num plain">{formatWinRate(column.line?.survivalRate)}</td>
                {/each}
              </tr>
            </tbody>
          </table>
        {/each}
        {#if stats.shipState === "unavailable"}
          <p class="note">Wargaming did not return ship statistics for this player. Press Refresh to try again.</p>
        {:else if stats.shipState === "ready" && stats.ship === null}
          <p class="note">This player has no random battles in this ship yet.</p>
        {/if}
        <p class="note">
          Solo leaves out battles played in a division. Div 2 and Div 3 count only battles in a division of two or three
          players. Personal rating uses expected values from wows-numbers.com.
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
    width: min(100vw, 26rem);
    max-width: 100vw;
    height: 100%;
    max-height: 100%;
    margin: 0;
    padding: 0;
    border: 0;
    border-left: 1px solid var(--border);
    background: var(--surface-raised);
    color: var(--text);
    box-shadow: var(--shadow-pop);
  }

  .sheet,
  .sheet::backdrop {
    transition:
      opacity var(--duration-base) var(--ease-out),
      translate var(--duration-base) var(--ease-out),
      overlay var(--duration-base) allow-discrete,
      display var(--duration-base) allow-discrete;
  }

  .sheet {
    opacity: 0;
    translate: var(--space-5) 0;
  }

  .sheet[open] {
    opacity: 1;
    translate: 0 0;
  }

  .sheet::backdrop {
    background: color-mix(in srgb, var(--bg) 55%, transparent);
    opacity: 0;
  }

  .sheet[open]::backdrop {
    opacity: 1;
  }

  @starting-style {
    .sheet[open] {
      opacity: 0;
      translate: var(--space-5) 0;
    }

    .sheet[open]::backdrop {
      opacity: 0;
    }
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    min-height: 100%;
    padding: var(--space-5);
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .heading {
    display: grid;
    gap: var(--space-1);
    min-width: 0;
  }

  .relation {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--fs-xl);
    font-weight: var(--weight-strong);
    letter-spacing: var(--tracking-display);
    overflow-wrap: anywhere;
  }

  .clan {
    color: var(--text-faint);
    font-weight: var(--weight-regular);
  }

  .ship {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-md);
  }

  .class-name {
    margin-left: var(--space-1);
  }

  .lines {
    width: 100%;
    border-collapse: collapse;
  }

  .lines caption {
    padding-bottom: var(--space-2);
    font-size: var(--fs-md);
    text-align: left;
  }

  .lines th {
    padding: 0 var(--cell-pad);
    border-bottom: 1px solid var(--border);
    color: var(--text-muted);
    font-size: var(--fs-sm);
    font-weight: var(--weight-regular);
    text-align: left;
    white-space: nowrap;
  }

  .lines th:first-child {
    padding-left: 0;
  }

  .lines thead th {
    padding-bottom: var(--space-2);
    border-bottom-color: var(--border-strong);
    font-size: var(--fs-xs);
    text-align: right;
  }

  .lines thead td {
    border-bottom: 1px solid var(--border-strong);
  }

  .lines tr {
    height: var(--row-height);
  }

  .lines :global(td) {
    font-size: var(--fs-md);
  }

  .lines :global(td:last-child) {
    padding-right: 0;
  }

  .plain {
    padding: 0 var(--cell-pad);
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
    color: var(--text-muted);
    font-size: var(--fs-md);
  }

  .with-icon {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
  }

  .with-icon :global(svg) {
    flex: none;
    margin-top: 0.125em;
  }

  .error {
    color: var(--text);
  }

  .error :global(svg) {
    color: var(--danger);
  }

  .links {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin-top: auto;
    padding-top: var(--space-4);
    border-top: 1px solid var(--border);
  }

  h3 {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
    font-weight: var(--weight-regular);
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
