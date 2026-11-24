<script lang="ts">
  import { MISSING, formatStat, placeholderFor, type StatKind } from "../../lib/format";
  import { isTiered, tierColor, tierName, type Tier } from "../../lib/tiers";

  interface Props {
    kind: StatKind;
    label: string;
    value?: number | null;
    tier?: Tier | null;
    rated?: boolean;
    colored?: boolean;
    pending?: boolean;
  }

  let {
    kind,
    label,
    value = null,
    tier = null,
    rated = isTiered(kind),
    colored = false,
    pending = false,
  }: Props = $props();

  const text = $derived(formatStat(kind, value));
  const spoken = $derived.by(() => {
    if (text === MISSING) {
      return "No data";
    }
    return rated ? `${text}, ${tierName(tier)}` : text;
  });
  const color = $derived(colored && rated && tier ? tierColor(tier) : undefined);
</script>

{#if pending}
  <td class="num stat" aria-label="Loading" title="{label}: loading">
    <span class="skeleton" aria-hidden="true"><span class="sample">{placeholderFor(kind)}</span></span>
  </td>
{:else}
  <td class="num stat" class:missing={text === MISSING} style:color title="{label}: {spoken}" aria-label={spoken}>
    {text}
  </td>
{/if}

<style>
  .stat {
    width: 1%;
    padding: 0 var(--space-1);
    border-bottom: 1px solid var(--border);
    text-align: right;
    white-space: nowrap;
  }

  .stat:last-child {
    padding-right: var(--space-3);
  }

  .missing {
    color: var(--text-faint);
  }

  .skeleton {
    display: inline-block;
    line-height: 1;
    border-radius: var(--radius-sm);
    background: var(--surface-hover);
    vertical-align: middle;
    animation: pulse calc(var(--duration-base) * 4) var(--ease-out) infinite alternate;
  }

  .sample {
    visibility: hidden;
  }

  @keyframes pulse {
    from {
      opacity: 1;
    }

    to {
      opacity: 0.45;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .skeleton {
      animation: none;
    }
  }
</style>
