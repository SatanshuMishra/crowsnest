<script lang="ts">
  import { tick } from "svelte";
  import { backend } from "../../lib/backend";
  import { app } from "../../lib/state.svelte";
  import type { Region } from "../../lib/types";
  import { describeVerifyFailure, validateApplicationId } from "../../lib/validation";
  import Button from "../ui/Button.svelte";
  import Icon from "../ui/Icon.svelte";

  type Status =
    | { kind: "idle" }
    | { kind: "verifying" }
    | { kind: "clearing" }
    | { kind: "saved" }
    | { kind: "cleared" }
    | { kind: "invalid"; message: string }
    | { kind: "failed"; message: string };

  const id = $props.id();
  let draft = $state(app.settings?.applicationId ?? "");
  let status = $state<Status>({ kind: "idle" });
  let input = $state<HTMLInputElement>();

  const savedId = $derived(app.settings?.applicationId ?? null);
  const optional = $derived(app.info?.hasBundledApplicationId ?? false);
  const region = $derived<Region>(app.settings?.regionOverride ?? app.snapshot?.install?.region ?? "eu");
  const working = $derived(status.kind === "verifying" || status.kind === "clearing");
  const problem = $derived(status.kind === "invalid" || status.kind === "failed" ? status.message : null);
  const showsSavedId = $derived(
    status.kind === "idle" && savedId !== null && draft.trim().toLowerCase() === savedId,
  );

  function onInput() {
    if (!working) {
      status = { kind: "idle" };
    }
  }

  async function verifyAndSave(event: SubmitEvent) {
    event.preventDefault();
    if (working) {
      return;
    }
    const checked = validateApplicationId(draft);
    if (!checked.ok) {
      const message =
        checked.problem === "empty" && optional
          ? "Enter an ID to verify it, or leave the field empty to use the built-in ID."
          : checked.message;
      status = { kind: "invalid", message };
      return;
    }
    draft = checked.value;
    status = { kind: "verifying" };
    try {
      await backend.verifyApplicationId(checked.value, region);
    } catch (error) {
      status = { kind: "failed", message: describeVerifyFailure(error).message };
      return;
    }
    const saved = await app.update({ applicationId: checked.value });
    status = saved
      ? { kind: "saved" }
      : { kind: "failed", message: "Wargaming accepted the ID, but Crowsnest could not save it. Try again." };
  }

  async function clear() {
    status = { kind: "clearing" };
    const saved = await app.update({ applicationId: null });
    if (saved) {
      draft = "";
      status = { kind: "cleared" };
    } else {
      status = { kind: "failed", message: "Crowsnest could not clear the ID. Try again." };
    }
    await tick();
    input?.focus();
  }
</script>

<form class="field" novalidate onsubmit={verifyAndSave}>
  <label class="label" for="{id}-input">
    Application ID
    {#if optional}
      <span class="optional">(optional)</span>
    {/if}
  </label>
  <p class="description" id="{id}-description">
    Copy the 32-character ID from your application's page.
    {#if optional}
      Crowsnest uses its built-in ID while this field is empty.
    {/if}
  </p>
  <div class="row">
    <input
      bind:this={input}
      id="{id}-input"
      class="input"
      type="text"
      bind:value={draft}
      oninput={onInput}
      readonly={working}
      autocomplete="off"
      autocapitalize="off"
      spellcheck="false"
      aria-invalid={problem !== null}
      aria-describedby="{id}-description {id}-status"
    />
    <Button type="submit" variant="primary" busy={status.kind === "verifying"} disabled={working}>
      {status.kind === "verifying" ? "Verifying…" : "Verify and save"}
    </Button>
    {#if optional && savedId}
      <Button variant="ghost" busy={status.kind === "clearing"} disabled={working} onclick={clear}>Clear</Button>
    {/if}
  </div>
  <div class="status" id="{id}-status" aria-live="polite">
    {#if problem}
      <p class="message error"><Icon name="error" /><span>{problem}</span></p>
    {:else if status.kind === "verifying"}
      <p class="message muted"><Icon name="refresh" /><span>Checking the ID with Wargaming…</span></p>
    {:else if status.kind === "saved"}
      <p class="message success"><Icon name="check" /><span>Verified and saved.</span></p>
    {:else if status.kind === "cleared"}
      <p class="message muted"><Icon name="check" /><span>Cleared. Crowsnest now uses its built-in ID.</span></p>
    {:else if showsSavedId}
      <p class="message muted"><Icon name="check" /><span>This ID is saved and in use.</span></p>
    {/if}
  </div>
</form>

<style>
  .field {
    display: grid;
    gap: var(--space-1);
  }

  .label {
    font-size: var(--fs-md);
  }

  .optional {
    color: var(--text-muted);
  }

  .description {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: stretch;
    gap: var(--space-2);
    margin-top: var(--space-1);
  }

  .input {
    flex: 1 1 calc(var(--space-6) * 8);
    min-width: 0;
    min-height: 32px;
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--text-faint);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text);
    font-family: var(--font-mono);
    font-size: var(--fs-md);
  }

  .input[aria-invalid="true"] {
    border-color: var(--danger);
  }

  .input[readonly] {
    color: var(--text-muted);
  }

  .status:empty {
    display: none;
  }

  .message {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    margin: 0;
    font-size: var(--fs-sm);
  }

  .muted {
    color: var(--text-muted);
  }

  .success {
    color: var(--success);
  }

  .error {
    color: var(--danger);
  }
</style>
