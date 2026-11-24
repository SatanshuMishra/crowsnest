<script lang="ts">
  import { onMount } from "svelte";
  import { updates } from "../lib/updates.svelte";
  import Button from "./ui/Button.svelte";
  import Icon from "./ui/Icon.svelte";
  import IconButton from "./ui/IconButton.svelte";

  const update = $derived(updates.state);

  const announcement = $derived.by(() => {
    switch (update.status) {
      case "available":
        return update.error
          ? "The update could not be installed. Try again."
          : `Crowsnest ${update.version} is available.`;
      case "downloading":
        return `Downloading Crowsnest ${update.version}.`;
      case "ready":
        return update.error ?? `Crowsnest ${update.version} is installed. Restart to update.`;
      default:
        return "";
    }
  });

  onMount(() => {
    updates.startAutomaticCheck();
  });
</script>

<div class="update-notice">
  <span class="visually-hidden" aria-live="polite">{announcement}</span>
  {#if update.status === "available"}
    <div class="pill">
      {#if update.error}
        <span class="text error" title={update.error}>
          <Icon name="error" />
          Update failed
        </span>
      {:else}
        <span class="text">
          <Icon name="download" />
          <span>Version <span class="num">{update.version}</span> is available</span>
        </span>
      {/if}
      <Button variant="primary" size="sm" onclick={() => updates.installUpdate()}>
        {update.error ? "Try again" : "Install"}
      </Button>
      <IconButton icon="close" label="Dismiss update" onclick={() => updates.dismiss()} />
    </div>
  {:else if update.status === "downloading"}
    <div class="pill trailing-text">
      <span class="text">Downloading <span class="num">{update.version}</span></span>
      <div
        class="bar"
        class:indeterminate={update.percent === null}
        role="progressbar"
        aria-label="Update download"
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={update.percent ?? undefined}
      >
        <span class="fill" style:width={update.percent === null ? undefined : `${update.percent}%`}></span>
      </div>
      {#if update.percent !== null}
        <span class="num">{update.percent}%</span>
      {/if}
    </div>
  {:else if update.status === "ready"}
    <div class="pill" class:trailing-text={update.error !== null}>
      {#if update.error}
        <span class="text error">
          <Icon name="error" />
          Close and reopen Crowsnest to finish updating
        </span>
      {:else}
        <span class="text">
          <Icon name="check" />
          <span>Version <span class="num">{update.version}</span> is installed</span>
        </span>
        <Button variant="primary" size="sm" onclick={() => updates.restart()}>Restart to update</Button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .update-notice {
    display: contents;
  }

  .pill {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    min-height: var(--target-min);
    padding: 0 var(--space-1) 0 var(--space-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
    background: var(--surface-raised);
    color: var(--text);
    font-size: var(--fs-sm);
    white-space: nowrap;
  }

  .pill.trailing-text {
    padding-right: var(--space-3);
  }

  .text {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .error {
    color: var(--danger);
  }

  .bar {
    position: relative;
    width: calc(var(--space-6) * 3);
    height: var(--space-2);
    overflow: hidden;
    border-radius: var(--radius-sm);
    background: var(--border-strong);
  }

  .fill {
    position: absolute;
    inset: 0 auto 0 0;
    width: 0;
    background: var(--accent);
    transition: width var(--duration-base) var(--ease-out);
  }

  .indeterminate .fill {
    width: 40%;
    animation: sweep calc(var(--duration-base) * 6) var(--ease-out) infinite;
  }

  @keyframes sweep {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(250%);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .indeterminate .fill {
      animation: none;
    }
  }
</style>
