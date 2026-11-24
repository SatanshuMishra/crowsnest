<script lang="ts" module>
  import type { Wash } from "../lib/intro";

  const WHATS_NEW = "What\u2019s new";
  const CAPTION = "Each choice applies straight away.";
  const STEP_WASHES: readonly Wash[] = ["teal", "violet", "green"];

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
  import OptionCards from "./appearance/OptionCards.svelte";
  import PaletteSwatch from "./appearance/PaletteSwatch.svelte";
  import Button from "./ui/Button.svelte";
  import Icon from "./ui/Icon.svelte";
  import IconButton from "./ui/IconButton.svelte";
  import {
    APPEARANCE,
    optionFor,
    type AppearanceKey,
    type AppearanceOption,
    type AppearanceValues,
  } from "../lib/appearance";
  import { APPEARANCE_STEPS, onboardingPages, type Page, type SetupAudience, type SetupOutcome } from "../lib/intro";
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
  const canSkip = untrack(() => audience !== "tour");
  const returnFocus = document.activeElement;

  let index = $state(0);
  let direction = $state<"forward" | "back">("forward");
  let announcement = $state("");
  let caption = $state<string | null>(null);
  let closing = $state<"finished" | "closed" | null>(null);
  let card = $state<HTMLDivElement>();
  let heading = $state<HTMLHeadingElement>();
  let primary = $state<HTMLButtonElement>();

  const page = $derived<Page | undefined>(pages[index]);
  const hero = $derived(page?.kind === "feature" ? page.feature.hero : null);
  const wash = $derived<Wash>(
    page?.kind === "feature" ? page.feature.wash : (STEP_WASHES[(page?.number ?? 1) - 1] ?? "teal"),
  );
  const mark = $derived(page?.kind === "appearance" && page.step.keys.includes("highlight"));
  const isFirst = $derived(index === 0);
  const isLast = $derived(index === pages.length - 1);
  const changed = $derived(APPEARANCE_KEYS.some((key) => settings[key] !== opening[key]));

  $effect(() => focusOnLeave(returnFocus));

  $effect(() => {
    void tick().then(() => {
      const choice = card?.querySelector<HTMLElement>(".options [aria-checked=true]:not(:disabled)");
      (choice ?? primary)?.focus({ preventScroll: true });
    });
  });

  function kickerFor(current: Page): string {
    return current.kind === "feature" ? WHATS_NEW : `Appearance, step ${current.number} of ${APPEARANCE_STEPS.length}`;
  }

  function titleFor(current: Page): string {
    return current.kind === "feature" ? current.feature.title : current.step.title;
  }

  function goTo(next: number) {
    const target = pages[next];
    if (!target) {
      return;
    }
    direction = next > index ? "forward" : "back";
    caption = null;
    index = next;
    announcement = `Page ${next + 1} of ${pages.length}: ${titleFor(target)}`;
    void tick().then(() => {
      if (!card?.contains(document.activeElement)) {
        heading?.focus({ preventScroll: true });
      }
    });
  }

  function choose(key: AppearanceKey, value: AppearanceValues[AppearanceKey], option: AppearanceOption<unknown>) {
    caption = `${APPEARANCE[key].label}: ${option.label}.`;
    void app.update({ [key]: value } as Partial<Settings>);
  }

  function close(result: "finished" | "closed") {
    if (closing === null) {
      closing = result;
      onclosing?.();
    }
  }

  function onanimationend(event: AnimationEvent) {
    if (closing && event.animationName.endsWith("card-out")) {
      onclose({ changed });
    }
  }
</script>

<div class="layer">
  <div class="scrim" class:leaving={closing !== null}></div>
  <div
    bind:this={card}
    class="card"
    class:wide={page?.kind === "appearance"}
    class:leaving={closing !== null}
    role="dialog"
    aria-modal="true"
    aria-labelledby="{id}-title"
    aria-describedby={page?.kind === "feature" ? `${id}-body` : `${id}-kicker`}
    tabindex="-1"
    {@attach modalKeys(() => close("closed"), () => heading)}
    {onanimationend}
  >
    <div class="close">
      <IconButton icon="close" label="Close" onclick={() => close("closed")} />
    </div>
    <p class="visually-hidden" aria-live="polite">{announcement}</p>

    {#key index}
      {#if page?.kind === "appearance"}
        <div class="page steps" class:back={direction === "back"}>
          <header
            class="head wash"
            style:--wash="var(--wash-{wash})"
            style:--wash-2="var(--wash-{WASH_PARTNER[wash]})"
          >
            <p class="kicker" id="{id}-kicker">{kickerFor(page)}</p>
            <h2 id="{id}-title" tabindex="-1" bind:this={heading}>{page.step.title}</h2>
          </header>
          <div class="split">
            <section class="options" aria-labelledby="{id}-title">
              {#each page.step.keys as key (key)}
                <OptionCards
                  {key}
                  {settings}
                  theme={app.theme}
                  onchange={(value, option) => choose(key, value, option)}
                />
              {/each}
            </section>
            <aside class="preview" aria-labelledby="{id}-preview">
              <div class="preview-head">
                <h3 id="{id}-preview">Preview</h3>
                <span class="note">Sample team</span>
              </div>
              <AppearancePreview {settings} theme={app.theme} {mark} />
              <p class="caption" aria-live="polite">{caption ?? CAPTION}</p>
            </aside>
          </div>
        </div>
      {:else if page}
        <div class="page intro" class:back={direction === "back"}>
          <div
            class="hero wash"
            style:--wash="var(--wash-{wash})"
            style:--wash-2="var(--wash-{WASH_PARTNER[wash]})"
          >
            {#if hero?.kind === "palettes"}
              <ul class="palettes" aria-hidden="true">
                {#each PALETTES as palette (palette)}
                  <li>
                    <span>{optionFor("palette", palette).label}</span>
                    <PaletteSwatch {palette} theme={app.theme} />
                  </li>
                {/each}
              </ul>
            {:else if hero?.kind === "glyphs"}
              <div class="glyphs" aria-hidden="true">
                {#each hero.icons as icon (icon)}
                  <span class="tile"><Icon name={icon} size={32} /></span>
                {/each}
              </div>
            {/if}
          </div>
          <div class="content">
            <p class="kicker" id="{id}-kicker">{kickerFor(page)}</p>
            <h2 id="{id}-title" tabindex="-1" bind:this={heading}>{titleFor(page)}</h2>
            {#if page.kind === "feature"}
              <p class="body" id="{id}-body">{page.feature.body}</p>
            {/if}
          </div>
        </div>
      {/if}
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
        {:else if canSkip}
          <Button variant="ghost" size="lg" onclick={() => close("closed")}>Skip</Button>
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
    transition: width var(--duration-base) var(--ease-out);
    animation: card-in var(--duration-base) var(--ease-out);
  }

  .card.wide {
    width: min(var(--onboarding-wide-width), 100%);
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

  .wash {
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

  .hero {
    position: relative;
    flex: 1 1 auto;
    min-height: var(--onboarding-hero);
    overflow: hidden;
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
    flex: none;
    padding: var(--space-4) var(--space-6) var(--space-2);
  }

  p {
    margin: 0;
  }

  .kicker,
  .note,
  .caption {
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .kicker {
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

  .head {
    flex: none;
    padding: var(--space-5) var(--space-6) var(--space-4);
  }

  .steps h2 {
    font-size: var(--fs-xl);
  }

  .split {
    display: flex;
    flex: 1;
    min-height: 0;
    border-block: 1px solid var(--border);
  }

  .options {
    display: flex;
    flex: none;
    flex-direction: column;
    gap: var(--space-4);
    width: var(--onboarding-options-width);
    padding: var(--space-4) var(--space-5) var(--space-4) var(--space-6);
    overflow-y: auto;
  }

  .preview {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: var(--space-3);
    min-width: 0;
    padding: var(--space-4) var(--space-6) var(--space-4) var(--space-5);
    overflow-y: auto;
    border-left: 1px solid var(--border);
  }

  .preview-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-3);
  }

  h3 {
    margin: 0;
    font-size: var(--fs-sm);
    font-weight: var(--weight-strong);
  }

  .caption {
    min-height: 1.5em;
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

    .head {
      padding-block: var(--space-4) var(--space-3);
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

    .card,
    .dots span {
      transition: none;
    }
  }
</style>
