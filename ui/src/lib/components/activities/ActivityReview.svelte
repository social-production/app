<script lang="ts">
  import { locale, t } from '#lib/i18n/index.ts';
  import {
    namedRoles,
    placeSummary,
    type ActivityForm,
    type ActivityStepType,
  } from '#lib/domain/activityForm.ts';

  /** Step 4 (Activity creation mockup §4): each answer with Edit, like plans, and the
   * optional note for people coming. */
  interface Props {
    form: ActivityForm;
    stageLabel: string | null;
    onEdit: (step: ActivityStepType) => void;
    onNote: (note: string) => void;
    error: string;
  }

  let { form, stageLabel, onEdit, onNote, error }: Props = $props();

  const when = $derived.by(() => {
    if (!form.scheduledAt || !form.endsAt) return '—';
    const start = new Date(form.scheduledAt);
    const day = start.toLocaleDateString(locale(), {
      weekday: 'short',
      day: 'numeric',
      month: 'short',
    });
    const time = (d: Date): string =>
      d.toLocaleTimeString(locale(), { hour: 'numeric', minute: '2-digit' });
    return `${day} · ${time(start)} – ${time(new Date(form.endsAt))}`;
  });
  const who = $derived(
    namedRoles(form.roles)
      .map((r) => `${r.maximum ? `${r.required}–${r.maximum}` : `${r.required}+`} ${r.label}`)
      .join(' · ') || '—',
  );
</script>

{#snippet row(label: string, value: string, step: ActivityStepType)}
  <button type="button" onclick={() => onEdit(step)}>
    <small>{label}</small>
    <b>{value}</b>
    <u>{t('Edit')}</u>
  </button>
{/snippet}

<div class="activity-rev">
  {@render row(t('Name'), form.title || '—', 'when')}
  {@render row(t('When'), when, 'when')}
  {@render row(t('Where'), placeSummary(form) || '—', 'where')}
  {#if stageLabel}{@render row(t('Part of'), t(stageLabel), 'where')}{/if}
  {@render row(t('Who'), who, 'who')}
</div>
<label class="activity-field">
  <span>{t('Note for people coming (optional)')}</span>
  <textarea
    rows="3"
    value={form.note}
    placeholder={t('What should happen in this activity?')}
    oninput={(e) => onNote(e.currentTarget.value)}></textarea>
</label>
{#if error}<p class="validation-copy" role="alert">{error}</p>{/if}
