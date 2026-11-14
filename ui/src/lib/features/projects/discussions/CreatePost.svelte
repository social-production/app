<script lang="ts">
  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import {
    compressChatPhoto,
    rejectOutgoingAttachment,
  } from '#lib/components/chat/attachmentLimits.ts';
  import { createPost, POST_PHOTOS_MAX } from '#lib/services/discussions.ts';
  import { explain } from '#lib/services/errors.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import '#lib/components/styles/ux/photos.css';

  /**
   * web's CreatePostPage: one public note for your personal feed (ADR 0070), with up to four
   * photos (ADR 0101), each resized to fit 1 MB as chat photos are.
   */
  let body = $state(page.url.searchParams.get('prefill')?.trim() ?? '');
  let submitting = $state(false);
  let statusMessage = $state('');
  let photos = $state<{ file: File; url: string }[]>([]);
  let picker = $state<HTMLInputElement>();

  async function pick(): Promise<void> {
    const files = [...(picker?.files ?? [])];
    if (picker) picker.value = '';
    statusMessage = '';
    for (const file of files) {
      if (photos.length >= POST_PHOTOS_MAX) {
        statusMessage = `A post carries up to ${POST_PHOTOS_MAX} photos.`;
        break;
      }
      const rejection = file.type.startsWith('image/')
        ? rejectOutgoingAttachment(file)
        : 'Use a JPEG, PNG, or WebP photo.';
      if (rejection) {
        statusMessage = rejection;
        continue;
      }
      try {
        const small = await compressChatPhoto(file);
        photos = [...photos, { file: small, url: URL.createObjectURL(small) }];
      } catch (e) {
        statusMessage = e instanceof Error ? e.message : 'Could not process image.';
      }
    }
  }

  function drop(index: number): void {
    const gone = photos[index];
    if (gone) URL.revokeObjectURL(gone.url);
    photos = photos.filter((_, i) => i !== index);
  }

  async function publish(): Promise<void> {
    if (!body.trim() || submitting) return;
    submitting = true;
    statusMessage = '';
    try {
      const id = await createPost(
        body,
        photos.map((p) => p.file),
      );
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
    {#if photos.length > 0}
      <div class="post-photo-picks">
        {#each photos as photo, i (photo.url)}
          <div class="post-photo-pick">
            <img src={photo.url} alt={`Photo ${i + 1}`} />
            <button type="button" aria-label={`Remove photo ${i + 1}`} onclick={() => drop(i)}
              >×</button
            >
          </div>
        {/each}
      </div>
    {/if}
    <input
      bind:this={picker}
      type="file"
      accept="image/jpeg,image/png,image/webp"
      multiple
      hidden
      aria-label="Photos for this post"
      onchange={() => void pick()}
    />
    <button
      class="post-photo-add"
      type="button"
      disabled={photos.length >= POST_PHOTOS_MAX || submitting}
      onclick={() => picker?.click()}
    >
      <svg aria-hidden="true" viewBox="0 0 24 24"
        ><rect
          x="3.5"
          y="5.5"
          width="17"
          height="13"
          rx="2.5"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
        /><circle cx="8.5" cy="10" r="1.4" fill="currentColor" /><path
          d="m20.5 15.5-4.5-4.5-7.500 7.500"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linejoin="round"
        /></svg
      >
      Add photos
    </button>
    {#if statusMessage}<p class="status-note">{statusMessage}</p>{/if}
  </div>
</CreateSheet>
