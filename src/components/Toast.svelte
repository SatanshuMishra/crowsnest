<script lang="ts">
  import Icon from "./ui/Icon.svelte";
  import { toasts, type Toast } from "../lib/toast.svelte";

  let leaving = $state<number | null>(null);
  let focusBefore: HTMLElement | null = null;

  const toast = $derived(toasts.current);
  const politeText = $derived(toast && toast.tone !== "error" ? `${toast.title}. ${toast.body}` : "");
  const alertText = $derived(toast && toast.tone === "error" ? `${toast.title}. ${toast.body}` : "");

  function close(shown: Toast) {
    leaving = shown.id;
  }

  function rememberFocus(event: FocusEvent) {
    const from = event.relatedTarget;
    if (from instanceof HTMLElement && !(event.currentTarget as HTMLElement).contains(from)) {
      focusBefore = from;
    }
  }

  function onanimationend(event: AnimationEvent, shown: Toast) {
    if (event.animationName.endsWith("toast-out")) {
      const held = (event.currentTarget as HTMLElement).contains(document.activeElement);
      if (held && focusBefore?.isConnected) {
        focusBefore.focus({ preventScroll: true });
      }
      focusBefore = null;
      leaving = null;
      toasts.dismiss(shown.id);
    }
  }

  function escapeCloses(shown: Toast) {
    return (node: HTMLElement) => {
      const onkeydown = (event: KeyboardEvent) => {
        if (event.key === "Escape") {
          event.stopPropagation();
          close(shown);
        }
      };
      node.addEventListener("keydown", onkeydown);
      return () => node.removeEventListener("keydown", onkeydown);
    };
  }

  function act(shown: Toast) {
    close(shown);
    shown.action?.run();
  }
</script>

<div class="visually-hidden" aria-live="polite" aria-atomic="true">{politeText}</div>
<div class="visually-hidden" role="alert" aria-atomic="true">{alertText}</div>

{#if toast}
  {#key toast.id}
    <section
      class="toast"
      class:leaving={leaving === toast.id}
      class:error={toast.tone === "error"}
      aria-label="Notification"
      {@attach escapeCloses(toast)}
      onfocusin={rememberFocus}
      onanimationend={(event) => onanimationend(event, toast)}
    >
      <div class="content">
        <span class="icon"><Icon name={toast.tone === "error" ? "error" : "check"} /></span>
        <div class="text">
          <p class="title">{toast.title}</p>
          <p class="body">{toast.body}</p>
          {#if toast.action}
            <button type="button" class="action" onclick={() => act(toast)}>{toast.action.label}</button>
          {/if}
        </div>
        <button type="button" class="dismiss" aria-label="Dismiss" title="Dismiss" onclick={() => close(toast)}>
          <Icon name="close" size={14} />
        </button>
      </div>
      {#if toast.tone !== "error"}
        <span class="bar" aria-hidden="true" onanimationend={() => close(toast)}></span>
      {/if}
    </section>
  {/key}
{/if}

<style>
  .toast {
    position: fixed;
    right: var(--space-4);
    bottom: var(--space-4);
    z-index: 50;
    width: min(var(--toast-width), calc(100vw - var(--space-4) * 2));
    overflow: hidden;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
    background: var(--surface-raised);
    box-shadow: var(--shadow-pop);
    animation: toast-in var(--duration-base) var(--ease-out);
  }

  .leaving {
    animation: toast-out var(--duration-exit) var(--ease-in) forwards;
  }

  @keyframes toast-in {
    from {
      opacity: 0;
      translate: 0 var(--space-2);
    }
  }

  @keyframes toast-out {
    to {
      opacity: 0;
      translate: 0 var(--space-2);
    }
  }

  .content {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-2) var(--space-3) var(--space-3);
  }

  .icon {
    display: inline-flex;
    flex: none;
    margin-top: 0.125em;
    color: var(--success);
  }

  .error .icon {
    color: var(--danger);
  }

  .text {
    flex: 1;
    min-width: 0;
  }

  p {
    margin: 0;
  }

  .title {
    font-size: var(--fs-md);
    font-weight: var(--weight-strong);
  }

  .body {
    margin-top: 2px;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .action {
    min-height: var(--target-min);
    margin-top: var(--space-2);
    padding: 0 var(--space-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: transparent;
    font-size: var(--fs-sm);
    font-weight: var(--weight-medium);
    cursor: pointer;
  }

  .dismiss {
    display: inline-grid;
    flex: none;
    place-items: center;
    width: var(--target-min);
    height: var(--target-min);
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
  }

  .action:hover,
  .dismiss:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .bar {
    display: block;
    height: 2px;
    background: var(--text);
    transform-origin: left center;
    animation: deplete var(--toast-life) linear forwards;
  }

  .toast:hover .bar,
  .toast:focus-within .bar {
    animation-play-state: paused;
  }

  @keyframes deplete {
    from {
      transform: scaleX(1);
    }

    to {
      transform: scaleX(0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .bar {
      animation-duration: var(--toast-life) !important;
    }
  }
</style>
