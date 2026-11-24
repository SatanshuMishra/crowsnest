<script lang="ts">
  import { tick } from "svelte";
  import PlayerDetail from "../components/monitor/PlayerDetail.svelte";
  import StatusView from "../components/monitor/StatusView.svelte";
  import TeamTable from "../components/monitor/TeamTable.svelte";
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

  async function closePlayer() {
    const row = selection?.row;
    selection = null;
    await tick();
    if (row?.isConnected) {
      row.focus();
    }
  }
</script>

{#if snapshot && settings}
  <div class="monitor">
    <main class="content" inert={selectedPlayer !== null}>
      <StatusView {snapshot} {settings} {battleShown} />
      {#if battle && battleShown}
        {#key battle.id}
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
        {/key}
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
    gap: var(--space-5);
    position: relative;
    --scroll-inset-top: var(--space-5);
    min-height: 0;
    padding: var(--scroll-inset-top) var(--space-5) var(--space-4);
    overflow: auto;
  }

  .teams {
    display: grid;
    flex: 1;
    animation: teams-in calc(var(--duration-base) * 1.25) var(--ease-out);
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-6);
    min-height: 0;
  }

  @keyframes teams-in {
    from {
      opacity: 0;
      translate: 0 var(--space-1);
    }
  }

  @media (width < 1100px) {
    .teams {
      flex: none;
      grid-template-columns: minmax(0, 1fr);
      gap: var(--space-6);
    }
  }
</style>
