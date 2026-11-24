<script lang="ts">
  import { onMount } from "svelte";
  import { updateAnnouncement, updates } from "../lib/updates.svelte";
  import Button from "./ui/Button.svelte";
  import Icon from "./ui/Icon.svelte";
  import IconButton from "./ui/IconButton.svelte";

  const update = $derived(updates.state);
  const announcement = $derived(updateAnnouncement(update));
  const trailingText = $derived(
    update.status === "downloading" || (update.status === "ready" && update.error !== null),
  );

  let pill = $state<HTMLDivElement>();

  function install() {
    pill?.focus();
    void updates.installUpdate();
  }

  onMount(() => {
    updates.startAutomaticCheck();
  });
</script>

<div class="update-notice">
  <span class="visually-hidden" aria-live="polite">{announcement}</span>
  {#if update.status === "available" || update.status === "downloading" || update.status === "ready"}
    <div
      bind:this={pill}
      class="pill"
      class:trailing-text={trailingText}
      tabindex="-1"
      role="group"
      aria-label="Crowsnest update"
    >
      {#if update.status === "available"}
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
        <Button variant="primary" size="sm" onclick={install}>
          {update.error ? "Try again" : "Install"}
        </Button>
        <IconButton icon="close" label="Dismiss update" onclick={() => updates.dismiss()} />
      {:else if update.status === "downloading"}
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
      {:else if update.status === "ready"}
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
    gap: var(--space-3);
    min-height: var(--target-min);
    margin-right: var(--space-2);
    color: var(--text-muted);
    font-size: var(--fs-sm);
    white-space: nowrap;
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
    height: 2px;
    overflow: hidden;
    background: var(--border-strong);
  }

  .fill {
    position: absolute;
    inset: 0 auto 0 0;
    width: 0;
    background: var(--text);
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
