<script lang="ts">
  import { TIERS, tierColor } from "../../lib/tiers";
  import type { Palette } from "../../lib/types";

  interface Props {
    palette: Palette;
    theme: "dark" | "light";
  }

  let { palette, theme }: Props = $props();
</script>

<span class="swatch" data-theme={theme} data-palette={palette} aria-hidden="true">
  {#each TIERS as tier (tier)}
    <span style:background={tierColor(tier)}></span>
  {/each}
</span>

<style>
  .swatch {
    display: grid;
    flex: none;
    grid-template-columns: repeat(8, var(--swatch-cell, 9px));
    gap: var(--swatch-gap, 2px);
  }

  .swatch span {
    height: var(--swatch-height, 14px);
    border-radius: var(--swatch-radius, 2px);
  }
</style>
