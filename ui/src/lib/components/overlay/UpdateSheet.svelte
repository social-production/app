<script lang="ts">
  import VoteKicker from '#lib/components/governance/VoteKicker.svelte';
  import { t } from '#lib/i18n/index.ts';

  import OverlaySheet from './OverlaySheet.svelte';

  import '#lib/components/styles/governance.css';
  import '#lib/components/styles/ux/updatesheet.css';

  /**
   * web's AddUpdateSheet in the one vote sheet's style (owner, 2026-11-18): the kind line,
   * "What's new?", a plain writing area and one line saying members vote first. The update is
   * a proposal: it shows once members approve it.
   */
  interface Props {
    open: boolean;
    pending: boolean;
    message: string | undefined;
    projectTitle: string;
    onSubmit: (body: string) => void;
    onClose: () => void;
  }

  let { open, pending, message, projectTitle, onSubmit, onClose }: Props = $props();

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
  title={t("What's new?")}
  labelledById="project-add-update-sheet"
  rootClass="update-sheet"
  {onClose}
>
  {#snippet kicker()}<VoteKicker label={t('Update')} meta={projectTitle} />{/snippet}
  <form id="project-add-update-sheet-form" class="update-form" onsubmit={submit}>
    {#if message}<div class="warning-card" role="alert">{message}</div>{/if}
    <textarea
      bind:value={body}
      rows="8"
      aria-label={t("What's new?")}
      placeholder={t('Twelve bikes fixed on Saturday…')}></textarea>
    <p class="update-vote">{t('Members vote before it appears.')}</p>
  </form>
  {#snippet footer()}
    <div class="sheet-actions">
      <button
        class="sheet-submit"
        disabled={pending || !body.trim()}
        form="project-add-update-sheet-form"
        type="submit">{pending ? t('Working...') : t('Send to vote')}</button
      >
    </div>
  {/snippet}
</OverlaySheet>
