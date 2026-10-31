<script lang="ts">
  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import { createThread } from '#lib/services/discussions.ts';
  import { explain } from '#lib/services/errors.ts';
  import ScopeTagsStep from '../scopes/ScopeTagsStep.svelte';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  /** web's CreateThreadPage: Basics (title, opening post) → Tags (ADR 0070). */
  const steps = [
    { id: 'basics', title: 'Basics' },
    { id: 'scope', title: 'Tags' },
  ];

  let step = $state(0);
  let title = $state('');
  let body = $state('');
  let tags = $state<string[]>([]);
  let submitting = $state(false);
  let statusMessage = $state('');

  const missing = $derived(
    step === 0
      ? [!title.trim() && 'thread title', !body.trim() && 'opening post'].filter(
          (m): m is string => !!m,
        )
      : tags.length === 0
        ? ['a primary tag']
        : [],
  );

  async function publish(): Promise<void> {
    if (submitting) return;
    submitting = true;
    statusMessage = '';
    try {
      const id = await createThread(title, body, tags);
      await goto(resolve('/threads/[id]', { id }));
    } catch (e) {
      statusMessage = explain(e);
    } finally {
      submitting = false;
    }
  }
</script>

<CreateSheet
  title="Create thread"
  description="A thread is a discussion people can follow and reply to. Tag it to a channel or community so it shows up where that topic already lives."
  {steps}
  stepIndex={step}
  {missing}
  submitLabel="Publish thread"
  {submitting}
  onStep={(i) => (step = i)}
  onSubmit={() => void publish()}
  onClose={() => void goto(resolve('/'))}
>
  {#if step === 0}
    <div class="form-stack">
      <label>
        <RequiredLabel>Thread title</RequiredLabel>
        <input bind:value={title} maxlength="200" aria-required="true" />
      </label>
      <label>
        <RequiredLabel>Opening post</RequiredLabel>
        <textarea bind:value={body} rows="5" aria-required="true"></textarea>
      </label>
    </div>
  {:else}
    <div class="form-stack">
      <ScopeTagsStep {tags} onChange={(t) => (tags = t)} helperText="Required." choosePrimary />
      {#if tags.length === 0}
        <p class="status-note">Add one primary channel or community tag.</p>
      {/if}
      {#if statusMessage}<p class="status-note">{statusMessage}</p>{/if}
    </div>
  {/if}
</CreateSheet>
