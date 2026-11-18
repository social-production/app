<script lang="ts">
  import KindGlyph from '#lib/components/icons/KindGlyph.svelte';
  import { kindColor, kindOf } from '#lib/domain/kindGlyphs.ts';
  import type { CharterMode } from '#lib/domain/projects.ts';
  import { plural, t } from '#lib/i18n/index.ts';

  import OverlaySheet from './OverlaySheet.svelte';

  import '#lib/components/styles/governance.css';
  import '#lib/components/styles/ux/updatesheet.css';

  /**
   * web's AddUpdateSheet, modernised (owner, 2026-11-18, option A): the project it goes to as a
   * card, "What changed?" in a soft field, and the reminder that members vote it in. The update
   * is a proposal: it shows once members approve it.
   */
  interface Props {
    open: boolean;
    pending: boolean;
    message: string | undefined;
    project: { title: string; mode: CharterMode; members: number };
    onSubmit: (body: string) => void;
    onClose: () => void;
  }

  let { open, pending, message, project, onSubmit, onClose }: Props = $props();

  let body = $state('');

  const TYPES: Partial<Record<CharterMode, string>> = {
    proposal: 'Proposal',
    productive: 'Project',
    'collective-service': 'Service',
    event: 'Event',
    'help-request': 'Request',
  };
  const typeLabel = $derived(TYPES[project.mode] ?? 'Project');

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
  <form id="project-add-update-sheet-form" class="update-form" onsubmit={submit}>
    <div class="update-project">
      <span class="update-glyph" style={`--surface-accent: ${kindColor(kindOf(project.mode))}`}>
        <KindGlyph kind={kindOf(project.mode)} size={22} />
      </span>
      <span class="update-project-copy">
        <b>{project.title}</b>
        <small>{t(typeLabel)} · {plural(project.members, '{n} member', '{n} members')}</small>
      </span>
    </div>
    {#if message}<div class="warning-card" role="alert">{message}</div>{/if}
    <label class="update-field">
      <span class="update-label">{t('What changed?')}</span>
      <textarea bind:value={body} rows="7" placeholder={t('Share what changed on this project...')}
      ></textarea>
    </label>
    <p class="update-vote">
      <svg aria-hidden="true" viewBox="0 0 24 24">
        <path
          d="M5 12.5 10 17.5 19 7.5"
          fill="none"
          stroke="currentColor"
          stroke-width="2.2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      {t('Members vote it in')}
    </p>
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
