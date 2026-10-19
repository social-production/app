<script lang="ts">
  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import { BODY_MAX, TITLE_MAX } from '#lib/domain/threads.ts';

  /**
   * web's CreateThreadPage, basics step only: the thread belongs to this project, so web's
   * channel/community tag step has nothing to choose.
   */
  interface Props {
    submitting: boolean;
    message: string;
    onSubmit: (title: string, body: string) => void;
    onClose: () => void;
  }

  let { submitting, message, onSubmit, onClose }: Props = $props();

  let title = $state('');
  let body = $state('');
  const missing = $derived([
    ...(title.trim() ? [] : ['Thread title']),
    ...(body.trim() ? [] : ['Opening post']),
  ]);
</script>

<CreateSheet
  title="Create thread"
  description="A thread is a discussion people can follow and reply to. It lives in this project, where its members and visitors can find it."
  steps={[{ id: 'basics', title: 'Basics' }]}
  stepIndex={0}
  {missing}
  submitLabel="Publish thread"
  {submitting}
  onStep={() => undefined}
  onSubmit={() => onSubmit(title, body)}
  {onClose}
>
  <div class="form-stack">
    <label>
      <RequiredLabel>Thread title</RequiredLabel>
      <input bind:value={title} maxlength={TITLE_MAX} aria-required="true" />
    </label>
    <label>
      <RequiredLabel>Opening post</RequiredLabel>
      <textarea bind:value={body} rows="5" maxlength={BODY_MAX} aria-required="true"></textarea>
    </label>
    {#if message}<p class="status-note">{message}</p>{/if}
  </div>
</CreateSheet>
