<script lang="ts" module>
  import type { AppearanceKey } from "../lib/appearance";

  interface Step {
    title: string;
    keys: readonly AppearanceKey[];
  }

  interface Intro {
    kicker: string;
    title: string;
    body: string;
    points: readonly { title: string; body: string }[];
    start: string;
    skip: string;
    note: string;
  }

  const STEPS: readonly Step[] = [
    { title: "Theme and size", keys: ["theme", "textSize", "density"] },
    { title: "Colour", keys: ["palette", "coloredValues"] },
    { title: "Rows", keys: ["highlight", "highlightStat", "showClanTags"] },
  ];

  const APPEARANCE_KEYS: readonly AppearanceKey[] = STEPS.flatMap((step) => step.keys);

  const INTRO: Intro = {
    kicker: "Welcome to Crowsnest",
    title: "A few display settings",
    body: "Crowsnest has eight appearance settings across three short steps, each with a live preview. The defaults work well, and every setting stays in Settings, Appearance.",
    points: [
      { title: "Theme and size", body: "Dark, light or System, the text size and how tall rows are." },
      { title: "Colour", body: "The tier palette, and whether numbers take tier colours." },
      { title: "Rows", body: "Whether each player is marked by tier, and which statistic sets the mark." },
    ],
    start: "Start",
    skip: "Use the defaults",
    note: "The default look",
  };
</script>

<script lang="ts">
  import { tick, untrack } from "svelte";
  import AppearancePreview from "./appearance/AppearancePreview.svelte";
  import OptionCards from "./appearance/OptionCards.svelte";
  import Button from "./ui/Button.svelte";
  import IconButton from "./ui/IconButton.svelte";
  import { APPEARANCE, type AppearanceOption, type AppearanceValues } from "../lib/appearance";
  import type { SetupOutcome } from "../lib/intro";
  import { focusOnLeave, modalKeys } from "../lib/modal";
  import { app } from "../lib/state.svelte";
  import type { Settings } from "../lib/types";

  interface Props {
    audience: "new" | "tour";
    settings: Settings;
    onclose: (outcome: SetupOutcome) => void;
  }

  let { audience, settings, onclose }: Props = $props();
  const id = $props.id();

  const opening = untrack(() => $state.snapshot(settings));
  const returnFocus = document.activeElement;

  let index = $state(untrack(() => audience) === "tour" ? 0 : -1);
  let caption = $state<string | null>(null);
  let closing = $state<"finished" | "closed" | null>(null);
  let heading = $state<HTMLHeadingElement>();

  const intro = $derived(audience === "new" ? INTRO : null);
  const step = $derived(index >= 0 ? STEPS[index] : null);
  const isLast = $derived(index === STEPS.length - 1);
  const kicker = $derived(step ? `Step ${index + 1} of ${STEPS.length}` : (intro?.kicker ?? ""));
  const title = $derived(step ? step.title : (intro?.title ?? ""));
  const note = $derived(step ? "Sample team" : (intro?.note ?? ""));
  const shownCaption = $derived(caption ?? (step ? "Each choice applies straight away." : ""));
  const changed = $derived(APPEARANCE_KEYS.some((key) => settings[key] !== opening[key]));

  $effect(() => focusOnLeave(returnFocus));

  $effect(() => {
    void index;
    void tick().then(() => heading?.focus({ preventScroll: true }));
  });

  function goTo(next: number) {
    caption = null;
    index = next;
  }

  function choose(key: AppearanceKey, value: AppearanceValues[AppearanceKey], option: AppearanceOption<unknown>) {
    caption = `${APPEARANCE[key].label}: ${option.label}.`;
    void app.update({ [key]: value } as Partial<Settings>);
  }

  function close(result: "finished" | "closed") {
    closing ??= result;
  }

  function onanimationend(event: AnimationEvent) {
    if (closing && event.animationName.endsWith("dialog-out")) {
      onclose({ changed, finished: closing === "finished" });
    }
  }
</script>

<div class="layer">
  <div class="scrim" class:leaving={closing !== null}></div>
  <div
    class="dialog"
    class:leaving={closing !== null}
    role="dialog"
    aria-modal="true"
    aria-labelledby="{id}-title"
    aria-describedby="{id}-kicker"
    tabindex="-1"
    {@attach modalKeys(() => close("closed"), () => heading)}
    {onanimationend}
  >
    <header class="head">
      <div class="heading">
        <p class="kicker" id="{id}-kicker">{kicker}</p>
        <h2 id="{id}-title" tabindex="-1" bind:this={heading}>{title}</h2>
      </div>
      <IconButton icon="close" label="Close" onclick={() => close("closed")} />
    </header>

    {#key index}
      <div class="body">
        <section class="options" aria-labelledby="{id}-title">
          {#if step}
            {#each step.keys as key (key)}
              <OptionCards
                {key}
                {settings}
                theme={app.theme}
                onchange={(value, option) => choose(key, value, option)}
              />
            {/each}
          {:else if intro}
            <p class="intro">{intro.body}</p>
            <ul class="points">
              {#each intro.points as point (point.title)}
                <li>
                  <span class="point-title">{point.title}</span>
                  <span class="point-body">{point.body}</span>
                </li>
              {/each}
            </ul>
          {/if}
        </section>
        <aside class="preview" aria-labelledby="{id}-preview">
          <div class="preview-head">
            <h3 id="{id}-preview">Preview</h3>
            <span class="note">{note}</span>
          </div>
          <AppearancePreview {settings} theme={app.theme} mark={index === STEPS.length - 1} />
          <p class="caption" aria-live="polite">{shownCaption}</p>
        </aside>
      </div>
    {/key}

    <footer class="foot">
      <div class="foot-start">
        {#if intro && !step}
          <Button variant="ghost" onclick={() => close("closed")}>{intro.skip}</Button>
        {:else}
          <span class="progress" aria-hidden="true">
            {#each STEPS as entry, position (entry.title)}
              <span class:done={position <= index}></span>
            {/each}
          </span>
        {/if}
      </div>
      <div class="foot-end">
        {#if step && !(audience === "tour" && index === 0)}
          <Button onclick={() => goTo(index - 1)}>Back</Button>
        {/if}
        <Button variant="primary" onclick={() => (isLast ? close("finished") : goTo(index + 1))}>
          {step ? (isLast ? "Finish" : "Next") : (intro?.start ?? "Start")}
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
    padding: var(--space-5);
  }

  .scrim {
    position: absolute;
    inset: 0;
    background: var(--scrim);
    animation: scrim-in var(--duration-base) var(--ease-out);
  }

  .dialog {
    position: relative;
    display: flex;
    flex-direction: column;
    width: min(var(--dialog-width), 100%);
    height: min(var(--dialog-height), 100%);
    overflow: hidden;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-xl);
    background: var(--surface-raised);
    box-shadow: var(--shadow-pop);
    animation: dialog-in var(--duration-base) var(--ease-out);
  }

  .scrim.leaving {
    animation: scrim-out var(--duration-exit) var(--ease-in) forwards;
  }

  .dialog.leaving {
    animation: dialog-out var(--duration-exit) var(--ease-in) forwards;
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

  @keyframes dialog-in {
    from {
      opacity: 0;
      transform: translateY(var(--space-2)) scale(0.98);
    }
  }

  @keyframes dialog-out {
    to {
      opacity: 0;
      transform: translateY(var(--space-1)) scale(0.98);
    }
  }

  .head {
    display: flex;
    flex: none;
    align-items: flex-start;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-3) var(--space-3) var(--space-5);
    border-bottom: 1px solid var(--border);
  }

  .heading {
    flex: 1;
    min-width: 0;
  }

  p {
    margin: 0;
  }

  .kicker,
  .note,
  .caption,
  .intro,
  .point-body {
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  h2 {
    margin: 2px 0 0;
    font-family: var(--font-display);
    font-size: var(--fs-xl);
    font-weight: var(--weight-strong);
    letter-spacing: var(--tracking-display);
  }

  h2:focus,
  .dialog:focus {
    outline: none;
  }

  h2:focus-visible {
    outline: 2px solid var(--focus);
    outline-offset: 4px;
  }

  .body {
    display: flex;
    flex: 1;
    min-height: 0;
    animation: body-in var(--duration-base) var(--ease-out);
  }

  @keyframes body-in {
    from {
      opacity: 0;
    }
  }

  .options {
    display: flex;
    flex: none;
    flex-direction: column;
    gap: var(--space-4);
    width: var(--dialog-options-width);
    padding: var(--space-4) var(--space-5);
    overflow-y: auto;
  }

  .intro {
    color: var(--text);
    font-size: var(--fs-md);
    line-height: 1.5;
  }

  .points {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .points li {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-left: var(--space-3);
    border-left: 2px solid var(--border-strong);
  }

  .point-title {
    font-weight: var(--weight-strong);
  }

  .preview {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: var(--space-3);
    min-width: 0;
    padding: var(--space-4) var(--space-5);
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
    padding: var(--space-3) var(--space-5);
    border-top: 1px solid var(--border);
  }

  .foot-end {
    display: flex;
    gap: var(--space-2);
  }

  .progress {
    display: flex;
    gap: var(--space-1);
  }

  .progress span {
    width: var(--space-5);
    height: 3px;
    border-radius: 2px;
    background: var(--border-strong);
    transition: background-color var(--duration-base) var(--ease-out);
  }

  .progress span.done {
    background: var(--text);
  }

  @media (prefers-reduced-motion: reduce) {
    .scrim,
    .dialog,
    .body {
      animation-duration: 1ms;
    }

    .scrim.leaving,
    .dialog.leaving {
      animation-duration: 1ms;
    }
  }
</style>
