<script lang="ts">
  import AppearancePreview from "../appearance/AppearancePreview.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import {
    APPEARANCE,
    isUnavailable,
    optionFor,
    resolveTheme,
    unavailableReason,
    type AppearanceKey,
    type AppearanceValues,
  } from "../../lib/appearance";
  import { placePopover } from "../../lib/popover";
  import { preview, type PreviewVia } from "../../lib/preview.svelte";
  import { app } from "../../lib/state.svelte";
  import type { Settings } from "../../lib/types";

  type Value = AppearanceValues[AppearanceKey];

  interface Props {
    key: AppearanceKey;
    settings: Settings;
    found?: boolean;
  }

  let { key, settings, found = false }: Props = $props();
  const id = $props.id();
  const tipId = `${id}-preview`;
  const EDGE_MARGIN = 16;

  let row = $state<HTMLDivElement>();

  const field = $derived(APPEARANCE[key]);
  const value = $derived<Value>(settings[key]);
  const isToggle = $derived(typeof value === "boolean");
  const unavailable = $derived(isUnavailable(key, settings));
  const description = $derived(unavailable ? (unavailableReason(key) ?? field.description) : field.description);
  const active = $derived(preview.current?.key === key && !unavailable ? preview.current : null);
  const shownValue = $derived<Value>(active ? active.value : value);
  const shown = $derived({ ...settings, [key]: shownValue } as Settings);
  const option = $derived(optionFor(key, shownValue as never));
  const caption = $derived(
    key === "theme" && shownValue === "system"
      ? `${option.caption} It is ${app.systemDark ? "dark" : "light"} on this computer.`
      : option.caption,
  );
  const usage = $derived(
    shownValue === value ? "In use now" : isToggle ? "Click the switch to use this" : "Click to use this",
  );
  const choices = $derived(field.options.map((choice) => ({ value: String(choice.value), label: choice.label })));

  function change(next: Value) {
    void app.update({ [key]: next } as Partial<Settings>);
  }

  function show(next: Value, via: PreviewVia) {
    if (!unavailable) {
      preview.show({ key, value: next, via });
    }
  }

  function onfocusout(event: FocusEvent) {
    if (!row?.contains(event.relatedTarget as Node | null)) {
      preview.end(key, "focus");
    }
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && active) {
      event.stopPropagation();
      preview.clear();
    }
  }

  function anchor(node: HTMLElement) {
    const host = row;
    const content = node.firstElementChild;
    if (!host || !content) {
      return;
    }
    const scroller = host.closest<HTMLElement>("[data-scroll-root]");
    const place = () => {
      const rect = host.getBoundingClientRect();
      node.style.maxHeight = "none";
      const placement = placePopover(
        { top: rect.top, bottom: rect.bottom, left: rect.left, right: rect.right },
        {
          width: window.innerWidth,
          height: window.innerHeight,
          top: scroller?.getBoundingClientRect().top ?? 0,
          margin: EDGE_MARGIN,
        },
        { width: node.offsetWidth, height: node.scrollHeight },
      );
      node.style.top = placement.top === null ? "auto" : `${placement.top}px`;
      node.style.bottom = placement.bottom === null ? "auto" : `${placement.bottom}px`;
      node.style.left = `${placement.left}px`;
      node.style.width = `${placement.width}px`;
      node.style.maxHeight = `${placement.maxHeight}px`;
      node.style.visibility = "visible";
    };
    const close = () => preview.clear();
    const observer = new ResizeObserver(place);
    observer.observe(content);
    place();
    scroller?.addEventListener("scroll", close, { passive: true });
    window.addEventListener("resize", close);
    return () => {
      observer.disconnect();
      scroller?.removeEventListener("scroll", close);
      window.removeEventListener("resize", close);
    };
  }
</script>

<div
  bind:this={row}
  data-setting={key}
  data-found={found || undefined}
  role="presentation"
  onmouseleave={() => preview.end(key, "hover")}
  {onfocusout}
  {onkeydown}
>
  {#if isToggle}
    <Toggle
      label={field.label}
      {description}
      checked={value === true}
      disabled={unavailable}
      onchange={(checked) => change(checked)}
      onpreview={(via) => show(!value, via)}
      describedBy={active ? tipId : undefined}
    />
  {:else}
    <Segmented
      label={field.label}
      {description}
      value={String(value)}
      options={choices}
      disabled={unavailable}
      onchange={(next) => change(next as Value)}
      onpreview={(next, via) => show(next as Value, via)}
      describedBy={(choice) => (active && String(active.value) === choice ? tipId : undefined)}
    />
  {/if}

  {#if active}
    <div class="popover" id={tipId} role="tooltip" {@attach anchor}>
      <div class="content">
        <div class="head">
          <p class="title">{field.label}: {option.label}</p>
          <p class="usage">{usage}</p>
        </div>
        <AppearancePreview
          settings={shown}
          theme={resolveTheme(shown.theme, app.systemDark)}
          mark={key === "highlight" || key === "highlightStat"}
        />
        <p class="caption">{caption}</p>
      </div>
    </div>
  {/if}
</div>

<style>
  .popover {
    position: fixed;
    z-index: 30;
    width: min(var(--popover-width), calc(100vw - var(--space-6)));
    overflow: auto;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
    background: var(--surface-raised);
    box-shadow: var(--shadow-pop);
    visibility: hidden;
    animation: popover-in var(--duration-fast) var(--ease-out);
  }

  @keyframes popover-in {
    from {
      opacity: 0;
    }
  }

  .content {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-3);
  }

  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-1) var(--space-3);
  }

  p {
    margin: 0;
  }

  .title {
    font-size: var(--fs-md);
    font-weight: var(--weight-strong);
  }

  .usage,
  .caption {
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }
</style>
