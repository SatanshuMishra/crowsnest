<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { backend } from "../../lib/backend";
  import { clientVersionLabel } from "../../lib/format";
  import { app } from "../../lib/state.svelte";
  import type { GameInstall, InstallSource } from "../../lib/types";
  import Button from "../ui/Button.svelte";
  import Icon from "../ui/Icon.svelte";

  type Task = "detect" | "choose" | "use" | "replays";

  interface Feedback {
    tone: "info" | "error";
    text: string;
  }

  const sourceLabels: Record<InstallSource, string> = {
    gameCenter: "Wargaming Game Center",
    steam: "Steam",
    defaultPath: "Default path",
    manual: "Chosen by you",
  };

  const NOT_AN_INSTALL =
    "That folder is not a World of Warships install. Choose the folder that holds WorldOfWarships.exe and the bin folder.";
  const SAVE_FAILED = "Crowsnest could not save this folder. Try again.";
  const PICKER_FAILED = "The folder picker could not open. Try again.";

  const id = $props.id();
  let task = $state<Task | null>(null);
  let feedback = $state<Feedback | null>(null);
  let replaysError = $state<string | null>(null);
  let candidates = $state<GameInstall[]>([]);
  let selected = $state<string | null>(null);

  const install = $derived(app.snapshot?.install ?? null);
  const replaysOverride = $derived(app.settings?.replaysDirOverride ?? null);
  const working = $derived(task !== null);

  async function pickFolder(title: string, defaultPath: string | undefined): Promise<string | null> {
    const path = await open({ directory: true, multiple: false, title, defaultPath });
    return typeof path === "string" ? path : null;
  }

  async function saveInstall(path: string): Promise<boolean> {
    const saved = await app.update({ installPath: path });
    feedback = saved ? null : { tone: "error", text: SAVE_FAILED };
    return saved;
  }

  async function detect() {
    task = "detect";
    feedback = null;
    candidates = [];
    try {
      const found = await backend.detectInstalls();
      const [first] = found;
      if (!first) {
        feedback = { tone: "info", text: "No install was found. Choose the game folder yourself." };
      } else if (found.length === 1) {
        if (await saveInstall(first.root)) {
          feedback = { tone: "info", text: "Found one install. Crowsnest now uses it." };
        }
      } else {
        candidates = found;
        selected = found.find((candidate) => candidate.root === install?.root)?.root ?? first.root;
      }
    } catch {
      feedback = { tone: "error", text: "Detection did not finish. Try again, or choose the game folder yourself." };
    } finally {
      task = null;
    }
  }

  async function useSelected() {
    const choice = candidates.find((candidate) => candidate.root === selected);
    if (!choice) {
      return;
    }
    task = "use";
    if (await saveInstall(choice.root)) {
      candidates = [];
    }
    task = null;
  }

  async function adopt(path: string) {
    const isInstall = await backend.inspectInstall(path).then(
      () => true,
      () => false,
    );
    if (!isInstall) {
      feedback = { tone: "error", text: NOT_AN_INSTALL };
      return;
    }
    candidates = [];
    await saveInstall(path);
  }

  async function chooseInstall() {
    task = "choose";
    feedback = null;
    try {
      const path = await pickFolder("Choose the World of Warships folder", install?.root);
      if (path) {
        await adopt(path);
      }
    } catch {
      feedback = { tone: "error", text: PICKER_FAILED };
    } finally {
      task = null;
    }
  }

  async function saveReplaysOverride(path: string | null) {
    const saved = await app.update({ replaysDirOverride: path });
    replaysError = saved ? null : SAVE_FAILED;
  }

  async function chooseReplays() {
    task = "replays";
    replaysError = null;
    try {
      const path = await pickFolder("Choose the replays folder", replaysOverride ?? install?.replaysDir);
      if (path) {
        await saveReplaysOverride(path);
      }
    } catch {
      replaysError = PICKER_FAILED;
    } finally {
      task = null;
    }
  }

  async function clearReplays() {
    task = "replays";
    await saveReplaysOverride(null);
    task = null;
  }
</script>

<div class="picker">
  {#if install}
    <dl class="details" aria-label="Current game install">
      <dt>Folder</dt>
      <dd class="path">{install.root}</dd>
      <dt>Found by</dt>
      <dd>{sourceLabels[install.source]}</dd>
      <dt>Client version</dt>
      <dd class="num">{install.clientVersion ? clientVersionLabel(install.clientVersion) : "Not known yet"}</dd>
      <dt>Replays folder</dt>
      <dd class="path">{install.replaysDir}</dd>
      <dt>Replays</dt>
      <dd class="flag" class:off={!install.replaysEnabled}>
        <Icon name={install.replaysEnabled ? "check" : "warning"} />
        {install.replaysEnabled ? "Turned on in the game" : "Turned off in the game"}
      </dd>
    </dl>
  {:else}
    <div class="empty">
      <Icon name="folder" size={20} />
      <div>
        <p class="empty-title">No game folder yet</p>
        <p>Detect the game again, or choose the folder that holds WorldOfWarships.exe.</p>
      </div>
    </div>
  {/if}

  <div class="actions">
    <Button
      variant={install || candidates.length > 0 ? "secondary" : "primary"}
      busy={task === "detect"}
      disabled={working}
      onclick={detect}
    >
      <Icon name="search" />
      {task === "detect" ? "Detecting…" : "Detect again"}
    </Button>
    <Button busy={task === "choose"} disabled={working} onclick={chooseInstall}>
      <Icon name="folder" />
      {task === "choose" ? "Choosing…" : "Choose folder"}
    </Button>
  </div>

  {#if candidates.length > 1}
    <fieldset class="candidates">
      <legend>{candidates.length} installs were found. Choose the one you play.</legend>
      {#each candidates as candidate (candidate.root)}
        <label class="candidate">
          <input type="radio" name="{id}-install" value={candidate.root} bind:group={selected} />
          <span class="candidate-text">
            <span class="path">{candidate.root}</span>
            <span class="meta">
              {sourceLabels[candidate.source]}{candidate.clientVersion ? `, version ${clientVersionLabel(candidate.clientVersion)}` : ""}
            </span>
          </span>
        </label>
      {/each}
      <div class="actions">
        <Button variant="primary" size="sm" busy={task === "use"} disabled={working} onclick={useSelected}>
          Use this install
        </Button>
        <Button variant="ghost" size="sm" disabled={working} onclick={() => (candidates = [])}>Cancel</Button>
      </div>
    </fieldset>
  {/if}

  <div class="feedback" aria-live="polite">
    {#if feedback}
      <p class="message {feedback.tone}">
        <Icon name={feedback.tone === "error" ? "error" : "info"} />
        <span>{feedback.text}</span>
      </p>
    {/if}
  </div>

  <div class="replays">
    <div class="replays-text">
      <span class="label">Replays folder</span>
      <p class="hint" id="{id}-replays-hint">
        Only needed when the game writes replays somewhere other than its own folder.
      </p>
      {#if replaysOverride}
        <p class="path">{replaysOverride}</p>
      {:else}
        <p class="hint">Not set. Crowsnest reads the replays folder the game reports.</p>
      {/if}
    </div>
    <div class="actions">
      <Button
        size="sm"
        aria-label="Choose replays folder"
        aria-describedby="{id}-replays-hint"
        busy={task === "replays"}
        disabled={working}
        onclick={chooseReplays}
      >
        Choose
      </Button>
      {#if replaysOverride}
        <Button size="sm" variant="ghost" aria-label="Clear replays folder" disabled={working} onclick={clearReplays}>
          Clear
        </Button>
      {/if}
    </div>
  </div>
  <div class="feedback" aria-live="polite">
    {#if replaysError}
      <p class="message error">
        <Icon name="error" />
        <span>{replaysError}</span>
      </p>
    {/if}
  </div>
</div>

<style>
  .picker {
    display: grid;
    gap: var(--space-3);
  }

  .details,
  .empty {
    margin: 0;
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface);
  }

  .details {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    align-items: baseline;
    gap: var(--space-2) var(--space-4);
    font-size: var(--fs-md);
  }

  dt {
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  dd {
    margin: 0;
    min-width: 0;
  }

  .path {
    margin: 0;
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    overflow-wrap: anywhere;
  }

  .flag {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .flag.off {
    color: var(--warning);
  }

  .empty {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    color: var(--text-muted);
  }

  .empty p {
    margin: 0;
    font-size: var(--fs-sm);
  }

  .empty .empty-title {
    color: var(--text);
    font-size: var(--fs-md);
    font-weight: 600;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .candidates {
    display: grid;
    gap: var(--space-1);
    margin: 0;
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
  }

  legend {
    padding: 0 var(--space-1);
    font-size: var(--fs-md);
    font-weight: 600;
  }

  .candidate {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    min-height: var(--target-min);
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .candidate:hover {
    background: var(--surface-hover);
  }

  .candidate input {
    flex: none;
    margin: var(--space-1) 0 0;
    accent-color: var(--accent);
  }

  .candidate-text {
    display: grid;
    min-width: 0;
  }

  .meta,
  .hint {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .candidates .actions {
    margin-top: var(--space-2);
  }

  .feedback:empty {
    display: none;
  }

  .message {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    margin: 0;
    font-size: var(--fs-sm);
  }

  .message.info {
    color: var(--text-muted);
  }

  .message.error {
    color: var(--danger);
  }

  .replays {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-2) var(--space-4);
    padding-top: var(--space-3);
    border-top: 1px solid var(--border);
  }

  .replays-text {
    display: grid;
    gap: var(--space-1);
    flex: 1 1 0;
    min-width: 0;
  }

  .label {
    font-size: var(--fs-md);
    font-weight: 600;
  }
</style>
