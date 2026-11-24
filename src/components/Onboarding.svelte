<script lang="ts" module>
  import type { Wash } from "../lib/intro";

  const WELCOME = {
    kicker: "Before your first battle",
    title: "Welcome to Crowsnest",
    body: "Crowsnest shows the statistics of every player in your battle. Take a minute to choose how the player table looks, or keep the defaults. Every choice stays in Settings, Appearance.",
  };

  const WHATS_NEW = "What\u2019s new";

  const WASH_PARTNER: Record<Wash, Wash> = {
    teal: "violet",
    violet: "teal",
    amber: "violet",
    green: "teal",
  };
</script>

<script lang="ts">
  import { tick, untrack } from "svelte";
  import AppearancePreview from "./appearance/AppearancePreview.svelte";
  import PaletteSwatch from "./appearance/PaletteSwatch.svelte";
  import SetupChoice from "./appearance/SetupChoice.svelte";
  import Button from "./ui/Button.svelte";
  import Icon from "./ui/Icon.svelte";
  import IconButton from "./ui/IconButton.svelte";
  import { optionFor, type AppearanceKey } from "../lib/appearance";
  import {
    APPEARANCE_STEPS,
    onboardingPages,
    skipLabel,
    type Hero,
    type Page,
    type SetupAudience,
    type SetupOutcome,
  } from "../lib/intro";
  import { focusOnLeave, modalKeys } from "../lib/modal";
  import { app } from "../lib/state.svelte";
  import type { Palette, Settings } from "../lib/types";

  interface Props {
    audience: SetupAudience;
    settings: Settings;
    before?: Settings | null;
    onclosing?: () => void;
    onclose: (outcome: SetupOutcome) => void;
  }

  let { audience, settings, before = null, onclosing, onclose }: Props = $props();
  const id = $props.id();

  const PALETTES: readonly Palette[] = ["muted", "vibrant", "friendly"];
  const APPEARANCE_KEYS: readonly AppearanceKey[] = APPEARANCE_STEPS.flatMap((step) => step.keys);

  const pages = untrack(() => onboardingPages(audience, settings.introSeen));
  const opening = untrack(() => before ?? $state.snapshot(settings));
  const returnFocus = document.activeElement;

  let index = $state(0);
  let direction = $state<"forward" | "back">("forward");
  let announcement = $state("");
  let closing = $state<"finished" | "closed" | null>(null);
  let card = $state<HTMLDivElement>();
  let heading = $state<HTMLHeadingElement>();
  let primary = $state<HTMLButtonElement>();

  const page = $derived<Page>(pages[index] ?? { kind: "welcome" });
  const text = $derived(describe(page));
  const hero = $derived<Hero>(page.kind === "feature" ? page.feature.hero : { kind: "table" });
  const wash = $derived<Wash>(
    page.kind === "feature" ? page.feature.wash : page.kind === "appearance" ? page.step.wash : "teal",
  );
  const keys = $derived<readonly AppearanceKey[]>(page.kind === "appearance" ? page.step.keys : []);
  const mark = $derived(keys.includes("highlight"));
  const isFirst = $derived(index === 0);
  const isLast = $derived(index === pages.length - 1);
  const changed = $derived(APPEARANCE_KEYS.some((key) => settings[key] !== opening[key]));
  const skip = $derived(skipLabel(audience, changed));

  $effect(() => focusOnLeave(returnFocus));

  $effect(() => {
    void tick().then(() => {
      const choice = card?.querySelector<HTMLElement>(".choices [aria-checked=true]:not(:disabled)");
      (choice ?? primary)?.focus({ preventScroll: true });
    });
  });

  function describe(current: Page): { kicker: string; title: string; body: string } {
    if (current.kind === "feature") {
      return {
        kicker: WHATS_NEW,
        title: current.feature.title,
        body: current.feature.body,
      };
    }
    if (current.kind === "appearance") {
      return {
        kicker: `Appearance, step ${current.number} of ${APPEARANCE_STEPS.length}`,
        title: current.step.title,
        body: current.step.body,
      };
    }
    return WELCOME;
  }

  function goTo(next: number) {
    const target = pages[next];
    if (!target) {
      return;
    }
    direction = next > index ? "forward" : "back";
    index = next;
    announcement = `Page ${next + 1} of ${pages.length}: ${describe(target).title}`;
    void tick().then(() => {
      if (!card?.contains(document.activeElement)) {
        heading?.focus({ preventScroll: true });
      }
    });
  }

  function close(result: "finished" | "closed") {
    if (closing === null) {
      closing = result;
      onclosing?.();
    }
  }

  function onanimationend(event: AnimationEvent) {
    if (closing && event.animationName.endsWith("card-out")) {
      onclose({ changed, finished: closing === "finished" });
    }
  }
</script>

<div class="layer">
  <div class="scrim" class:leaving={closing !== null}></div>
  <div
    bind:this={card}
    class="card"
    class:leaving={closing !== null}
    role="dialog"
    aria-modal="true"
    aria-labelledby="{id}-title"
    aria-describedby="{id}-body"
    tabindex="-1"
    {@attach modalKeys(() => close("closed"), () => heading)}
    {onanimationend}
  >
    <div class="close">
      <IconButton icon="close" label="Close" onclick={() => close("closed")} />
    </div>
    <p class="visually-hidden" aria-live="polite">{announcement}</p>

    {#key index}
      <div class="page" class:intro={page.kind !== "appearance"} class:back={direction === "back"}>
        <div
          class="hero"
          style:--wash="var(--wash-{wash})"
          style:--wash-2="var(--wash-{WASH_PARTNER[wash]})"
        >
          {#if hero.kind === "palettes"}
            <ul class="palettes" aria-hidden="true">
              {#each PALETTES as palette (palette)}
                <li>
                  <span>{optionFor("palette", palette).label}</span>
                  <PaletteSwatch {palette} theme={app.theme} />
                </li>
              {/each}
            </ul>
          {:else if hero.kind === "glyphs"}
            <div class="glyphs" aria-hidden="true">
              {#each hero.icons as icon (icon)}
                <span class="tile"><Icon name={icon} size={32} /></span>
              {/each}
            </div>
          {:else}
            <div class="window">
              <AppearancePreview {settings} theme={app.theme} {mark} />
            </div>
          {/if}
        </div>

        <div class="content">
          <p class="kicker">{text.kicker}</p>
          <h2 id="{id}-title" tabindex="-1" bind:this={heading}>{text.title}</h2>
          <p class="body" id="{id}-body">{text.body}</p>
          {#if keys.length > 0}
            <div class="choices">
              {#each keys as key (key)}
                <SetupChoice {key} {settings} />
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/key}

    <footer class="foot">
      <span class="dots" aria-hidden="true">
        {#each pages as entry, position (position)}
          <span class:current={position === index} class:seen={position < index} data-kind={entry.kind}></span>
        {/each}
      </span>
      <div class="actions">
        {#if !isFirst}
          <Button size="lg" onclick={() => goTo(index - 1)}>Back</Button>
        {:else if skip}
          <Button variant="ghost" size="lg" onclick={() => close("closed")}>{skip}</Button>
        {/if}
        <Button
          variant="primary"
          size="lg"
          bind:element={primary}
          onclick={() => (isLast ? close("finished") : goTo(index + 1))}
        >
          {isLast ? "Done" : "Continue"}
        </Button>
      </div>
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
    padding: var(--space-4);
  }

  .scrim {
    position: absolute;
    inset: 0;
    background: var(--scrim);
    animation: scrim-in var(--duration-base) var(--ease-out);
  }

  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    width: min(var(--onboarding-width), 100%);
    height: min(var(--onboarding-height), 100%);
    overflow: hidden;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-xl);
    background: var(--surface-raised);
    box-shadow: var(--shadow-pop);
    animation: card-in var(--duration-base) var(--ease-out);
  }

  .card:focus {
    outline: none;
  }

  .scrim.leaving {
    animation: scrim-out var(--duration-exit) var(--ease-in) forwards;
  }

  .card.leaving {
    animation: card-out var(--duration-exit) var(--ease-in) forwards;
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

  @keyframes card-in {
    from {
      opacity: 0;
      transform: translateY(var(--space-2)) scale(0.98);
    }
  }

  @keyframes card-out {
    to {
      opacity: 0;
      transform: translateY(var(--space-1)) scale(0.98);
    }
  }

  .close {
    position: absolute;
    top: var(--space-3);
    right: var(--space-3);
    z-index: 1;
  }

  .page {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-height: 0;
    animation: page-in var(--duration-base) var(--ease-out);
  }

  .page.back {
    animation-name: page-back;
  }

  @keyframes page-in {
    from {
      opacity: 0;
      translate: var(--space-4) 0;
    }
  }

  @keyframes page-back {
    from {
      opacity: 0;
      translate: calc(var(--space-4) * -1) 0;
    }
  }

  .hero {
    position: relative;
    flex: none;
    height: var(--onboarding-hero);
    overflow: hidden;
    background:
      radial-gradient(
        80% 110% at 12% 0%,
        color-mix(in srgb, var(--wash) var(--wash-strength), transparent),
        transparent 72%
      ),
      radial-gradient(
        70% 100% at 96% 12%,
        color-mix(in srgb, var(--wash-2) var(--wash-strength), transparent),
        transparent 72%
      );
  }

  .intro .hero {
    flex: 1 1 auto;
    height: auto;
    min-height: var(--onboarding-hero);
  }

  .window {
    position: absolute;
    right: var(--space-6);
    bottom: 0;
    left: var(--space-6);
    max-height: calc(100% - var(--space-6));
    overflow: hidden;
    mask-image: linear-gradient(to bottom, #000 calc(100% - 3rem), transparent);
  }

  .palettes {
    position: absolute;
    inset: 0;
    display: grid;
    align-content: center;
    justify-items: center;
    gap: var(--space-2);
    margin: 0;
    padding: var(--space-5);
    list-style: none;
    --swatch-cell: 1.5rem;
    --swatch-height: 1.125rem;
    --swatch-gap: 3px;
    --swatch-radius: 4px;
  }

  .palettes li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-5);
    width: min(28rem, 100%);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface-raised);
    font-size: var(--fs-sm);
    font-weight: var(--weight-medium);
  }

  .glyphs {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-4);
  }

  .tile {
    display: grid;
    place-items: center;
    width: var(--onboarding-tile);
    height: var(--onboarding-tile);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-xl);
    background: var(--surface-raised);
    color: var(--text);
    box-shadow:
      0 0 0 var(--space-3) color-mix(in srgb, var(--wash) 12%, transparent),
      0 0 0 var(--space-6) color-mix(in srgb, var(--wash) 6%, transparent),
      var(--shadow-pop);
  }

  .content {
    flex: 1 1 auto;
    min-height: 0;
    padding: var(--space-4) var(--space-6) var(--space-2);
    overflow-y: auto;
  }

  .intro .content {
    flex: none;
  }

  p {
    margin: 0;
  }

  .kicker {
    color: var(--text-muted);
    font-size: var(--fs-sm);
    font-weight: var(--weight-medium);
  }

  h2 {
    margin: var(--space-1) 0 0;
    font-family: var(--font-display);
    font-size: var(--fs-2xl);
    font-weight: var(--weight-strong);
    line-height: 1.2;
    letter-spacing: var(--tracking-display);
  }

  h2:focus {
    outline: none;
  }

  h2:focus-visible {
    outline: 2px solid var(--focus);
    outline-offset: 4px;
  }

  .body {
    max-width: var(--measure);
    margin-top: var(--space-2);
    color: var(--text-muted);
    font-size: var(--fs-md);
    line-height: 1.5;
  }

  .choices {
    display: grid;
    gap: var(--space-2);
    margin-top: var(--space-4);
  }

  .foot {
    display: flex;
    flex: none;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-4) var(--space-6) var(--space-5);
  }

  .actions {
    display: flex;
    gap: var(--space-2);
  }

  .dots {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .dots span {
    width: 6px;
    height: 6px;
    border-radius: 3px;
    background: var(--border-strong);
    transition:
      width var(--duration-base) var(--ease-out),
      background-color var(--duration-base) var(--ease-out);
  }

  .dots span.seen {
    background: var(--text-faint);
  }

  .dots span.current {
    width: 20px;
    background: var(--text);
  }

  @media (max-height: 640px) {
    .card {
      --onboarding-hero: 8.5rem;
    }

    .window {
      max-height: calc(100% - var(--space-4));
    }

    .content {
      padding-block: var(--space-3) 0;
    }

    .choices {
      gap: var(--space-1);
      margin-top: var(--space-3);
    }

    .foot {
      padding-block: var(--space-3) var(--space-4);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .scrim,
    .card,
    .page,
    .scrim.leaving,
    .card.leaving {
      animation-duration: 1ms;
    }

    .dots span {
      transition: none;
    }
  }
</style>
