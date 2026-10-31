<script lang="ts">
  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import { createPost } from '#lib/services/discussions.ts';
  import { explain } from '#lib/services/errors.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  /** web's CreatePostPage: one public note for your personal feed (ADR 0070). */
  let body = $state(page.url.searchParams.get('prefill')?.trim() ?? '');
  let submitting = $state(false);
  let statusMessage = $state('');

  async function publish(): Promise<void> {
    if (!body.trim() || submitting) return;
    submitting = true;
    statusMessage = '';
    try {
      const id = await createPost(body);
      await goto(resolve('/posts/[id]', { id }));
    } catch (e) {
      statusMessage = explain(e) || 'The post could not be created.';
    } finally {
      submitting = false;
    }
  }
</script>

<CreateSheet
  title="New post"
  description="A post is a short public note on your personal feed."
  steps={[{ id: 'post', title: 'Post' }]}
  stepIndex={0}
  missing={[]}
  canSubmit={body.trim().length > 0}
  submitLabel="Publish post"
  submittingLabel="Posting..."
  {submitting}
  onStep={() => undefined}
  onSubmit={() => void publish()}
  onClose={() => void goto(resolve('/'))}
>
  <div class="composer">
    <p class="audience-cue">Public</p>
    <label>
      <RequiredLabel>What's on your mind?</RequiredLabel>
      <textarea
        bind:value={body}
        rows="8"
        placeholder="Share a direct post to your personal timeline..."
        aria-required="true"></textarea>
    </label>
    {#if statusMessage}<p class="status-note">{statusMessage}</p>{/if}
  </div>
</CreateSheet>
