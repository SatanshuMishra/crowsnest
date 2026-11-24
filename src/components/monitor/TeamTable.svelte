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
    onopen: (player: Player, row: HTMLTableRowElement) => void;
  }

  let { side, title, players, settings, onopen }: Props = $props();
  const id = $props.id();

  let tableWidth = $state(0);
  const columns = $derived(statColumns(settings.density, tableWidth || Number.POSITIVE_INFINITY));
  const allColumns = $derived([...columns.ship, ...columns.account]);
  const suffix = $derived(settings.soloStats ? ", solo" : "");
</script>

<section class="team {side}" aria-labelledby="{id}-title">
  <TeamSummary {title} titleId="{id}-title" {players} {settings} />
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
            <th scope="col" class="num stat-head" title={column.name + suffix}>
              <span aria-hidden="true">{column.short}</span>
              <span class="visually-hidden">{column.name + suffix}</span>
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
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface);
    overflow: hidden;
  }

  .allies {
    --team-accent: var(--ally);
  }

  .enemies {
    --team-accent: var(--enemy);
  }

  .scroll {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    border-spacing: 0;
  }

  thead {
    position: sticky;
    top: 0;
    z-index: 1;
    background: var(--surface);
  }

  th {
    padding: var(--space-1) var(--space-2);
    border-bottom: 1px solid var(--border-strong);
    color: var(--text-muted);
    font-size: var(--fs-xs);
    font-weight: 600;
    white-space: nowrap;
    background: var(--surface);
  }

  .text-head {
    text-align: left;
    vertical-align: bottom;
  }

  .player-head {
    padding-left: var(--space-3);
  }

  .group {
    padding-bottom: 0;
    border-bottom-color: var(--border);
    color: var(--text-faint);
    font-weight: 600;
    text-align: center;
  }

  .stat-head {
    padding: var(--space-1);
    text-align: right;
  }

  .stat-head:last-child,
  .group:last-child {
    padding-right: var(--space-3);
  }

  .solo {
    margin-left: var(--space-1);
    padding: 0 var(--space-1);
    border-radius: var(--radius-sm);
    background: var(--surface-hover);
    color: var(--text);
  }

  .empty {
    padding: var(--space-4) var(--space-3);
    color: var(--text-muted);
    font-size: var(--fs-sm);
    text-align: center;
  }

  @media (width < 1100px) {
    .team {
      overflow: visible;
    }

    .scroll {
      overflow: visible;
    }
  }
</style>
