<script lang="ts">
  import StatusPill from '#lib/components/StatusPill.svelte';
  import { NOTE_MAX, type Heartbeat } from '#lib/domain/health.ts';
  import { listHeartbeats, ping, postHeartbeat } from '#lib/services/health.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  let pong = $state<string>();
  let beats = $state<Heartbeat[]>([]);
  let note = $state('');
  let error = $state<string>();

  const tone = $derived(
    connection.current.status === 'connected'
      ? 'ok'
      : connection.current.status === 'connecting'
        ? 'pending'
        : 'error',
  );
  const label = $derived(
    connection.current.status === 'connected'
      ? `Connected to ${connection.current.node}`
      : connection.current.status === 'connecting'
        ? 'Connecting…'
        : 'Offline',
  );

  async function run(task: () => Promise<void>): Promise<void> {
    error = undefined;
    try {
      await task();
    } catch (e) {
      error = String(e);
    }
  }

  const refresh = () =>
    run(async () => {
      pong = await ping();
      beats = await listHeartbeats();
    });

  const post = () =>
    run(async () => {
      await postHeartbeat(note.trim());
      note = '';
      beats = await listHeartbeats();
    });

  $effect(() => {
    void refresh();
  });
</script>

<section class="panel">
  <h1>Network health</h1>
  <StatusPill {tone} {label} />
  {#if pong}<p>Node replied: {pong}</p>{/if}
  {#if error}<p class="error">{error}</p>{/if}

  <form onsubmit={(e) => (e.preventDefault(), post())}>
    <input bind:value={note} maxlength={NOTE_MAX} placeholder="Leave a heartbeat note" />
    <button disabled={!note.trim()}>Post</button>
  </form>

  <p>{beats.length} heartbeat(s) visible from this node</p>
  <button onclick={refresh}>Refresh</button>
</section>

<style>
  .panel {
    max-width: 560px;
    margin: 24px auto;
    padding: var(--page-gutter);
    background: var(--panel);
    border: 1px solid var(--panel-border);
    border-radius: var(--radius-lg);
  }
  .error {
    color: var(--need-strong);
  }
  form {
    display: flex;
    gap: 8px;
    margin: 12px 0;
  }
  input {
    flex: 1;
  }
</style>
