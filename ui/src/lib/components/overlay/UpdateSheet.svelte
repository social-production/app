<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from './OverlaySheet.svelte';

  import '#lib/components/styles/governance.css';

  /** web's AddUpdateSheet. The update is a proposal: members vote it onto the project. */
  interface Props {
    open: boolean;
    pending: boolean;
    message: string | undefined;
    onSubmit: (body: string) => void;
    onClose: () => void;
  }

  let { open, pending, message, onSubmit, onClose }: Props = $props();

  let body = $state('');

  $effect(() => {
    if (open) body = '';
  });

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    onSubmit(body.trim());
  }
</script>

<OverlaySheet
  {open}
  title={t('Post an update')}
  labelledById="project-add-update-sheet"
  rootClass="update-sheet"
  {onClose}
>
  {#snippet subtitle()}
    <p class="sheet-lead">
      {t('Share what changed so people following this can see the latest status.')}
    </p>
  {/snippet}
  <form id="project-add-update-sheet-form" class="update-form" onsubmit={submit}>
    {#if message}<div class="warning-card" role="alert">{message}</div>{/if}
    <label class="field">
      <span class="sr-only">{t('Update')}</span>
      <textarea bind:value={body} rows="8" placeholder={t('Share what changed on this project...')}
      ></textarea>
    </label>
  </form>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={onClose}>{t('Cancel')}</button>
      <button
        class="sheet-submit"
        disabled={pending || !body.trim()}
        form="project-add-update-sheet-form"
        type="submit">{pending ? t('Working...') : t('Post update')}</button
      >
    </div>
  {/snippet}
</OverlaySheet>
