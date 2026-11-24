<script lang="ts">
  import PlayerDetail from "../components/monitor/PlayerDetail.svelte";
  import StatusView from "../components/monitor/StatusView.svelte";
  import TeamTable from "../components/monitor/TeamTable.svelte";
  import TopBar from "../components/monitor/TopBar.svelte";
  import { app } from "../lib/state.svelte";
  import type { Player } from "../lib/types";

  type Side = "allies" | "enemies";

  interface Selection {
    side: Side;
    name: string;
    row: HTMLElement;
  }

  let selection = $state.raw<Selection | null>(null);

  const snapshot = $derived(app.snapshot);
  const settings = $derived(app.settings);
  const battle = $derived(snapshot?.battle ?? null);
  const battleShown = $derived(
    battle !== null &&
      (snapshot?.phase === "waiting" || snapshot?.phase === "loading" || snapshot?.phase === "inBattle"),
  );
  const region = $derived(battle?.region ?? snapshot?.install?.region ?? null);
  const selectedPlayer = $derived.by(() => {
    if (!selection || !battle || !battleShown) {
      return null;
    }
    const team = selection.side === "allies" ? battle.allies : battle.enemies;
    const name = selection.name;
    return team.players.find((player) => player.name === name) ?? null;
  });

  function openPlayer(side: Side, player: Player, row: HTMLElement) {
    selection = { side, name: player.name, row };
  }

  function closePlayer() {
    const row = selection?.row;
    selection = null;
    if (row?.isConnected) {
      row.focus();
    }
  }
</script>

{#if snapshot && settings}
  <div class="monitor">
    <TopBar {snapshot} />
    <main class="content">
      <StatusView {snapshot} {settings} {battleShown} />
      {#if battle && battleShown}
        <div class="teams">
          <TeamTable
            side="allies"
            title="Allies"
            players={battle.allies.players}
            {settings}
            onopen={(player, row) => openPlayer("allies", player, row)}
          />
          <TeamTable
            side="enemies"
            title="Enemies"
            players={battle.enemies.players}
            {settings}
            onopen={(player, row) => openPlayer("enemies", player, row)}
          />
        </div>
      {/if}
    </main>
    <PlayerDetail player={selectedPlayer} {region} {settings} onclose={closePlayer} />
  </div>
{/if}

<style>
  .monitor {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .content {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: var(--space-2);
    min-height: 0;
    padding: var(--space-2) var(--space-3) var(--space-3);
    overflow: auto;
  }

  .teams {
    display: grid;
    flex: 1;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-3);
    min-height: 0;
  }

  @media (width < 1100px) {
    .teams {
      flex: none;
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
