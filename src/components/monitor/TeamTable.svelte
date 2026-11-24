<script lang="ts">
  import PlayerRow from "./PlayerRow.svelte";
  import TeamSummary from "./TeamSummary.svelte";
  import { statColumns } from "../../lib/team";
  import type { Player, Settings } from "../../lib/types";

  interface Props {
    side: "allies" | "enemies";
    title: string;
    players: Player[];
    settings: Settings;
    summary?: boolean;
    marked?: string | null;
    onopen: (player: Player, row: HTMLTableRowElement) => void;
  }

  let { side, title, players, settings, summary = true, marked = null, onopen }: Props = $props();
  const id = $props.id();

  let tableWidth = $state(0);
  const columns = $derived(statColumns(settings.density, tableWidth || Number.POSITIVE_INFINITY));
  const allColumns = $derived([...columns.ship, ...columns.account]);
  const suffix = $derived(settings.soloStats ? ", solo" : "");
</script>

<section
  class="team {side}"
  aria-labelledby={summary ? `${id}-title` : undefined}
  aria-label={summary ? undefined : title}
>
  {#if summary}
    <TeamSummary {title} titleId="{id}-title" {players} {settings} />
  {/if}
  <p id="{id}-hint" class="visually-hidden">Press Enter to open this player's details.</p>
  <div class="scroll" bind:clientWidth={tableWidth}>
    <table>
      <caption class="visually-hidden">{title}</caption>
      <colgroup span="2"></colgroup>
      <colgroup span={columns.ship.length}></colgroup>
      <colgroup span={columns.account.length}></colgroup>
      <thead>
        <tr class="groups">
          <th scope="col" rowspan="2" class="text-head player-head">Player</th>
          <th scope="col" rowspan="2" class="text-head">Ship</th>
          <th scope="colgroup" colspan={columns.ship.length} class="group">
            This ship{#if settings.soloStats}<span class="solo">Solo</span>{/if}
          </th>
          <th scope="colgroup" colspan={columns.account.length} class="group">
            Account{#if settings.soloStats}<span class="solo">Solo</span>{/if}
          </th>
        </tr>
        <tr>
          {#each allColumns as column (column.id)}
            <th scope="col" class="num stat-head" class:marked={column.id === marked} title={column.name + suffix}>
              <span aria-hidden="true">{column.short}</span>
              <span class="visually-hidden">{column.name + suffix}{column.id === marked ? ", sets the row highlight" : ""}</span>
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each players as player, index (index)}
          <PlayerRow {player} {columns} {settings} describedBy="{id}-hint" {onopen} />
        {:else}
          <tr>
            <td class="empty" colspan={2 + allColumns.length}>No players on this team.</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</section>

<style>
  .team {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .scroll {
    flex: 1;
    min-height: 0;
    margin: 0 calc(-1 * var(--cell-pad)) 0 calc(-1 * (var(--cell-pad) + var(--space-4)));
    padding-left: var(--space-4);
    overflow: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    border-spacing: 0;
  }

  thead {
    position: sticky;
    top: calc(var(--scroll-inset-top, 0px) * -1);
    z-index: 1;
    background: var(--table-surface, var(--bg));
  }

  th {
    padding: var(--space-1) var(--cell-pad);
    border-bottom: 1px solid var(--border-strong);
    background: var(--table-surface, var(--bg));
    color: var(--text-muted);
    font-size: var(--fs-xs);
    font-weight: var(--weight-regular);
    white-space: nowrap;
  }

  .text-head {
    padding-bottom: var(--space-2);
    text-align: left;
    vertical-align: bottom;
  }

  .group {
    padding-bottom: var(--space-1);
    border-bottom: 0;
    background: linear-gradient(var(--border-strong), var(--border-strong)) center bottom / calc(100% - var(--space-3)) 1px
      no-repeat var(--table-surface, var(--bg));
    text-align: center;
  }

  .stat-head {
    padding-bottom: var(--space-2);
    text-align: right;
  }

  .marked {
    color: var(--text);
    font-weight: var(--weight-strong);
    box-shadow: inset 0 -2px 0 var(--text);
  }

  .solo {
    margin-left: var(--space-1);
    color: var(--text-faint);
  }

  .solo::before {
    content: "·" / "";
    margin-right: var(--space-1);
  }

  .empty {
    padding: var(--space-4) var(--space-2);
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  @media (width < 1100px) {
    .scroll {
      overflow: visible;
    }
  }
</style>
