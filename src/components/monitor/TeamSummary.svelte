<script lang="ts">
  import { formatInteger, formatRating, formatWinRate } from "../../lib/format";
  import { averageName, coverageLabel, summarizeTeam } from "../../lib/team";
  import { personalRatingTier, tierColor, tierName, winRateTier, type Tier } from "../../lib/tiers";
  import type { Player, Settings } from "../../lib/types";

  interface Props {
    title: string;
    titleId: string;
    players: Player[];
    settings: Settings;
  }

  let { title, titleId, players, settings }: Props = $props();

  interface Figure {
    id: string;
    short: string;
    name: string;
    text: string;
    tier: Tier | null;
    rated: boolean;
  }

  const summary = $derived(summarizeTeam(players, settings.teamAverage, settings.soloStats));
  const method = $derived(averageName(settings.teamAverage));
  const suffix = $derived(settings.soloStats ? ", solo" : "");

  const figures = $derived<Figure[]>([
    {
      id: "rating",
      short: "PR",
      name: `Team personal rating in their ships${suffix}`,
      text: formatRating(summary.personalRating),
      tier: personalRatingTier(summary.personalRating),
      rated: true,
    },
    {
      id: "ship-wr",
      short: "WR",
      name: `Team win rate in their ships${suffix}`,
      text: formatWinRate(summary.shipWinRate),
      tier: winRateTier(summary.shipWinRate),
      rated: true,
    },
    {
      id: "account-wr",
      short: "Account WR",
      name: `Team account win rate${suffix}`,
      text: formatWinRate(summary.accountWinRate),
      tier: winRateTier(summary.accountWinRate),
      rated: true,
    },
    {
      id: "battles",
      short: "Battles",
      name: `Team account battles${suffix}`,
      text: formatInteger(summary.battles),
      tier: null,
      rated: false,
    },
  ]);

  function describe(figure: Figure): string {
    const value = figure.rated && figure.tier ? `${figure.text}, ${tierName(figure.tier)}` : figure.text;
    return `${figure.name} (${method.toLowerCase()}): ${value}`;
  }
</script>

<header class="summary">
  <h2 id={titleId}>{title}</h2>
  <dl>
    {#each figures as figure (figure.id)}
      <div class="figure" title={describe(figure)}>
        <dt><span aria-hidden="true">{figure.short}</span><span class="visually-hidden">{figure.name}</span></dt>
        <dd
          class="num"
          style:color={settings.coloredValues && figure.tier ? tierColor(figure.tier) : undefined}
        >
          {figure.text}{#if figure.rated && figure.tier}<span class="visually-hidden">, {tierName(figure.tier)}</span>{/if}
        </dd>
      </div>
    {/each}
  </dl>
  <p class="coverage" title={method}>
    {coverageLabel(summary)} <span class="method">· {settings.teamAverage}{settings.soloStats ? " · solo" : ""}</span>
  </p>
</header>

<style>
  .summary {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: baseline;
    gap: var(--space-2) var(--space-4);
    padding-bottom: var(--space-4);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--fs-xl);
    font-weight: var(--weight-strong);
    letter-spacing: var(--tracking-display);
  }

  dl {
    display: flex;
    grid-column: 1 / -1;
    grid-row: 2;
    flex-wrap: wrap;
    gap: var(--space-1) var(--space-4);
    margin: 0;
  }

  .figure {
    display: flex;
    align-items: baseline;
    gap: var(--space-1);
  }

  dt {
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  dd {
    margin: 0;
    font-size: var(--fs-md);
  }

  .coverage {
    grid-column: 2;
    grid-row: 1;
    margin: 0;
    color: var(--text-faint);
    font-size: var(--fs-sm);
  }
</style>
