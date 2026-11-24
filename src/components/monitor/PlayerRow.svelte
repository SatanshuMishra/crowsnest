<script lang="ts">
  import StatCell from "./StatCell.svelte";
  import { divisionLetter, romanNumeral, shipClassBadge } from "../../lib/format";
  import { columnValue, playerLines, type ColumnGroups } from "../../lib/team";
  import { highlightTier, statTier, tierColor } from "../../lib/tiers";
  import type { Player, Settings } from "../../lib/types";

  interface Props {
    player: Player;
    columns: ColumnGroups;
    settings: Settings;
    describedBy: string;
    onopen: (player: Player, row: HTMLTableRowElement) => void;
  }

  let { player, columns, settings, describedBy, onopen }: Props = $props();

  let row = $state<HTMLTableRowElement>();

  const stats = $derived(player.stats);
  const lines = $derived(playerLines(player.stats, settings.soloStats));
  const statCount = $derived(columns.ship.length + columns.account.length);
  const allColumns = $derived([...columns.ship, ...columns.account]);
  const suffix = $derived(settings.soloStats ? ", solo" : "");
  const isMe = $derived(player.relation === "me");
  const division = $derived(player.division === null ? null : divisionLetter(player.division));
  const badge = $derived(player.ship ? shipClassBadge(player.ship.class) : null);
  const highlight = $derived(
    settings.highlight !== "none" && lines ? highlightTier(settings.highlightStat, lines, player.expected) : null,
  );

  function open() {
    if (row) {
      onopen(player, row);
    }
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      open();
    }
  }
</script>

<tr
  bind:this={row}
  tabindex="0"
  class:me={isMe}
  class:edge={highlight !== null && settings.highlight === "edge"}
  class:fill={highlight !== null && settings.highlight === "fill"}
  style:--row-tier={highlight ? tierColor(highlight) : undefined}
  aria-busy={stats.kind === "pending" || undefined}
  aria-describedby={describedBy}
  onclick={open}
  {onkeydown}
>
  <th scope="row" class="player">
    <span class="clip">
      {#if settings.showClanTags && player.clanTag}
        <span class="clan">[{player.clanTag}]</span>
      {/if}
      <span class="name">{player.name}</span>
      {#if division}
        <span class="division" title="Division {division}" aria-hidden="true">({division})</span>
        <span class="visually-hidden">, division {division}</span>
      {/if}
      {#if isMe}
        <span class="visually-hidden">(You)</span>
      {/if}
    </span>
  </th>
  <td class="ship">
    {#if player.ship}
      <span class="clip">
        {#if badge}
          <span class="class-badge" title={badge.name} aria-hidden="true">{badge.short}</span>
          <span class="visually-hidden">{badge.name}</span>
        {/if}
        <span class="tier"><span class="visually-hidden">Tier </span>{romanNumeral(player.ship.tier)}</span>
        <span class="ship-name">{player.ship.name}</span>
      </span>
    {:else}
      <span class="clip unknown" title="Ship id {player.shipId}">Unknown ship</span>
    {/if}
  </td>
  {#if stats.kind === "pending"}
    {#each allColumns as column (column.id)}
      <StatCell kind={column.kind} label={column.name + suffix} pending />
    {/each}
  {:else if stats.kind === "bot"}
    <td class="message" colspan={statCount}>Bot</td>
  {:else if stats.kind === "hidden"}
    <td class="message" colspan={statCount}>Hidden profile</td>
  {:else if stats.kind === "notFound"}
    <td class="message" colspan={statCount}>Not found on Wargaming</td>
  {:else if stats.kind === "failed"}
    <td class="message" colspan={statCount} title={stats.reason}>
      Unavailable<span class="visually-hidden">: {stats.reason}</span>
    </td>
  {:else if lines}
    {#if lines.ship}
      {#each columns.ship as column (column.id)}
        {@const value = columnValue(column, lines)}
        <StatCell
          kind={column.kind}
          label={column.name + suffix}
          {value}
          tier={statTier(column.kind, value, player.expected)}
          colored={settings.coloredValues}
        />
      {/each}
    {:else if stats.kind === "loaded" && stats.shipState === "pending"}
      {#each columns.ship as column (column.id)}
        <StatCell kind={column.kind} label={column.name + suffix} pending />
      {/each}
    {:else if stats.kind === "loaded" && stats.shipState === "unavailable"}
      <td class="message" colspan={columns.ship.length} title="Wargaming did not return ship statistics. Press Refresh to try again.">
        Ship stats unavailable
      </td>
    {:else}
      <td class="message" colspan={columns.ship.length}>No battles in this ship</td>
    {/if}
    {#each columns.account as column (column.id)}
      {@const value = columnValue(column, lines)}
      <StatCell
        kind={column.kind}
        label={column.name + suffix}
        {value}
        tier={statTier(column.kind, value, null)}
        colored={settings.coloredValues}
      />
    {/each}
  {/if}
</tr>

<style>
  tr {
    height: var(--row-height);
    cursor: pointer;
  }

  tr > :global(*) {
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .me > :global(*) {
    background-color: var(--me-fill);
  }

  tr:hover > :global(*) {
    background-color: var(--surface-hover);
  }

  tr:focus-visible {
    outline: 2px solid var(--focus);
    outline-offset: -2px;
  }

  .me .name {
    font-weight: var(--weight-strong);
  }


  .fill > :global(*) {
    background-image: linear-gradient(
      color-mix(in srgb, var(--row-tier) var(--highlight-fill-alpha), transparent),
      color-mix(in srgb, var(--row-tier) var(--highlight-fill-alpha), transparent)
    );
  }

  .player {
    position: relative;
  }

  .edge .player::before {
    position: absolute;
    top: 50%;
    left: calc(-1 * (var(--space-2) + var(--pip-size) - var(--cell-pad)));
    width: var(--pip-size);
    height: var(--pip-size);
    border-radius: 1px;
    background: var(--row-tier);
    content: "";
    translate: 0 -50%;
  }

  th,
  td {
    padding: 0 var(--cell-pad);
    border-bottom: 1px solid var(--border);
    font-size: var(--fs-md);
    font-weight: var(--weight-regular);
    text-align: left;
    white-space: nowrap;
  }

  .player,
  .ship {
    max-width: 0;
    min-width: calc(var(--space-6) * 3);
  }

  .player {
    width: 53%;
  }

  .ship {
    width: 47%;
  }

  .clip {
    display: flex;
    align-items: baseline;
    gap: var(--cell-pad);
    min-width: 0;
    overflow: hidden;
  }

  .clip > :global(*) {
    flex: none;
  }

  .name,
  .ship-name {
    flex: 0 1 auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .clan {
    color: var(--text-faint);
    font-size: var(--fs-xs);
  }

  .division {
    color: var(--division);
  }

  .class-badge {
    order: 1;
    color: var(--text-faint);
    font-size: var(--fs-xs);
  }

  .tier {
    min-width: 1.3em;
    color: var(--text-muted);
    text-align: right;
  }

  .unknown {
    color: var(--text-faint);
  }

  .message {
    color: var(--text-faint);
    font-size: var(--fs-sm);
  }
</style>
