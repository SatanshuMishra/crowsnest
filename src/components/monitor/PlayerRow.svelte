<script lang="ts">
  import Icon from "../ui/Icon.svelte";
  import StatCell from "./StatCell.svelte";
  import { romanNumeral, shipClassBadge } from "../../lib/format";
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
    <td class="message" colspan={statCount}>
      <span class="message-line"><Icon name="eyeOff" size={14} />Hidden profile</span>
    </td>
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

  tr:hover > :global(*) {
    background-color: var(--surface-hover);
  }

  tr:focus-visible {
    outline: 2px solid var(--focus);
    outline-offset: -2px;
  }

  .me {
    outline: 2px solid var(--me);
    outline-offset: -2px;
  }

  .me .name {
    font-weight: 700;
  }

  .fill > :global(*) {
    background-image: linear-gradient(
      color-mix(in srgb, var(--row-tier) var(--highlight-fill-alpha), transparent),
      color-mix(in srgb, var(--row-tier) var(--highlight-fill-alpha), transparent)
    );
  }

  .edge .player {
    box-shadow: inset 3px 0 0 var(--row-tier);
  }

  th,
  td {
    padding: 0 var(--space-2);
    border-bottom: 1px solid var(--border);
    font-size: var(--fs-md);
    font-weight: 400;
    text-align: left;
    white-space: nowrap;
  }

  .player,
  .ship {
    width: 50%;
    max-width: 0;
    min-width: calc(var(--space-6) * 3);
  }

  .player {
    padding-left: var(--space-3);
  }

  .clip {
    display: flex;
    align-items: baseline;
    gap: var(--space-1);
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
    color: var(--text-muted);
  }

  .class-badge {
    padding: 0 var(--space-1);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    line-height: 1.4;
  }

  .tier {
    color: var(--text-muted);
    font-size: var(--fs-sm);
    font-weight: 600;
  }

  .unknown {
    color: var(--text-muted);
    font-style: italic;
  }

  .message {
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .message-line {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }
</style>
