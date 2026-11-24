<script lang="ts" module>
  import type { IconName } from "./ui/Icon.svelte";

  interface Feature {
    icon: IconName;
    title: string;
    body: string;
  }

  const FEATURES: readonly Feature[] = [
    {
      icon: "palette",
      title: "Colour palettes",
      body: "Show tiers as Muted, Vibrant or Colour-blind friendly.",
    },
    {
      icon: "eye",
      title: "Previews in Settings",
      body: "Point at an appearance option to see the player table in it before you choose.",
    },
    {
      icon: "pin",
      title: "Keep on top",
      body: "The pin in the title bar keeps Crowsnest above the game.",
    },
  ];
</script>

<script lang="ts">
  import logo from "../../assets/logo.svg";
  import Button from "./ui/Button.svelte";
  import Icon from "./ui/Icon.svelte";
  import { focusOnLeave, modalKeys } from "../lib/modal";

  interface Props {
    version: string | null;
    onclose: (review: boolean) => void;
  }

  let { version, onclose }: Props = $props();
  const id = $props.id();

  const returnFocus = document.activeElement;
  let closing = $state<"continue" | "review" | null>(null);
  let primary = $state<HTMLButtonElement>();

  $effect(() => {
    primary?.focus({ preventScroll: true });
  });

  $effect(() => focusOnLeave(returnFocus));

  function close(choice: "continue" | "review") {
    closing ??= choice;
  }

  function onanimationend(event: AnimationEvent) {
    if (closing && event.animationName.endsWith("sheet-out")) {
      onclose(closing === "review");
    }
  }
</script>

<div class="layer">
  <div class="scrim" class:leaving={closing !== null}></div>
  <div
    class="sheet"
    class:leaving={closing !== null}
    role="dialog"
    aria-modal="true"
    aria-labelledby="{id}-title"
    aria-describedby={version ? `${id}-version` : undefined}
    tabindex="-1"
    {@attach modalKeys(() => close("continue"), () => primary)}
    {onanimationend}
  >
    <header class="head">
      <img class="mark" src={logo} alt="" />
      <h2 id="{id}-title">What’s new in Crowsnest</h2>
      {#if version}
        <p class="version" id="{id}-version">Version {version}</p>
      {/if}
    </header>

    <ul class="features">
      {#each FEATURES as feature (feature.title)}
        <li>
          <span class="glyph"><Icon name={feature.icon} size={22} /></span>
          <span class="text">
            <span class="title">{feature.title}</span>
            <span class="body">{feature.body}</span>
          </span>
        </li>
      {/each}
    </ul>

    <footer class="foot">
      <Button variant="primary" bind:element={primary} onclick={() => close("continue")}>Continue</Button>
      <Button variant="ghost" size="sm" onclick={() => close("review")}>Review appearance</Button>
    </footer>
  </div>
</div>

<style>
  .layer {
    position: absolute;
    inset: 0;
    z-index: 20;
    display: grid;
    place-items: center;
    padding: var(--space-5);
  }

  .scrim {
    position: absolute;
    inset: 0;
    background: var(--scrim);
    animation: scrim-in var(--duration-base) var(--ease-out);
  }

  .sheet {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    width: min(var(--sheet-width), 100%);
    max-height: 100%;
    padding: var(--space-6) var(--space-6) var(--space-5);
    overflow-y: auto;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-xl);
    background: var(--surface-raised);
    box-shadow: var(--shadow-pop);
    animation: sheet-in var(--duration-base) var(--ease-out);
  }

  .sheet:focus {
    outline: none;
  }

  .scrim.leaving {
    animation: scrim-out var(--duration-exit) var(--ease-in) forwards;
  }

  .sheet.leaving {
    animation: sheet-out var(--duration-exit) var(--ease-in) forwards;
  }

  @keyframes scrim-in {
    from {
      opacity: 0;
    }
  }

  @keyframes scrim-out {
    to {
      opacity: 0;
    }
  }

  @keyframes sheet-in {
    from {
      opacity: 0;
      transform: scale(0.96);
    }
  }

  @keyframes sheet-out {
    to {
      opacity: 0;
      transform: scale(0.98);
    }
  }

  .head {
    display: grid;
    justify-items: center;
    gap: var(--space-1);
    text-align: center;
  }

  .mark {
    width: var(--sheet-mark);
    height: var(--sheet-mark);
    margin-bottom: var(--space-3);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--fs-xl);
    font-weight: var(--weight-strong);
    letter-spacing: var(--tracking-display);
  }

  .version {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .features {
    display: grid;
    gap: var(--space-5);
    margin: 0;
    padding: 0 var(--space-2);
    list-style: none;
  }

  li {
    display: grid;
    grid-template-columns: var(--space-6) 1fr;
    column-gap: var(--space-4);
    align-items: start;
  }

  .glyph {
    display: grid;
    place-items: center;
    padding-top: 1px;
    color: var(--text);
  }

  .text {
    display: grid;
    gap: 2px;
  }

  .title {
    font-size: var(--fs-md);
    font-weight: var(--weight-strong);
  }

  .body {
    color: var(--text-muted);
    font-size: var(--fs-sm);
    line-height: 1.45;
  }

  .foot {
    display: grid;
    justify-items: center;
    gap: var(--space-2);
  }

  .foot :global(.primary) {
    width: 100%;
    min-height: var(--sheet-action-height);
  }

  .foot :global(.ghost) {
    color: var(--text-muted);
  }

  @media (prefers-reduced-motion: reduce) {
    .scrim,
    .sheet,
    .scrim.leaving,
    .sheet.leaving {
      animation-duration: 1ms;
    }
  }
</style>
