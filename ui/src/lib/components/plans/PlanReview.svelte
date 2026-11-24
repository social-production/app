<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { scheduleSummary } from '#lib/domain/eventSchedule.ts';
  import { SOFTWARE_LICENSE_LABEL } from '#lib/domain/policy.ts';
  import type { PlanForm, PlanStep, PlanStepType } from '#lib/domain/planForm.ts';
  import { stagesSummary } from '#lib/domain/planStages.ts';
  import { REQUEST_MODES } from '#lib/domain/projects.ts';

  /**
   * The plan wizard's last step as one card (Edit, Plan and Meetup mockups §B, owner
   * 2026-11-24): a small label over each answer, and an Edit on each row that jumps back to
   * its step.
   */
  interface Props {
    form: PlanForm;
    steps: PlanStep[];
    onEdit: (step: PlanStepType) => void;
  }

  let { form, steps, onEdit }: Props = $props();

  const has = (id: PlanStepType): boolean => steps.some((s) => s.id === id);
  const placeStep = $derived<PlanStepType>(
    has('distribution-location') ? 'distribution-location' : 'location',
  );
  const requests = $derived(
    REQUEST_MODES.find((m) => m.value === form.requests)?.label ?? 'No requests for now',
  );
</script>

{#snippet row(label: string, value: string, step: PlanStepType)}
  <button type="button" onclick={() => onEdit(step)}>
    <small>{t(label)}</small>
    <b>{value}</b>
    <u>{t('Edit')}</u>
  </button>
{/snippet}

<div class="plan-rev">
  {@render row('Title', form.title, 'plan-overview')}
  {#if has('schedule-mode')}{@render row('When', scheduleSummary(form), 'schedule-mode')}{/if}
  {#if (has('location') || has('distribution-location')) && form.location}
    {@render row('Where', form.location, placeStep)}
  {/if}
  {#if has('distribution-location') && form.recipients.trim()}
    {@render row('Recipients', form.recipients, 'distribution-location')}
  {/if}
  {#if has('repository') && form.repositoryUrl}
    {@render row('Repository', `${form.repositoryUrl} · ${SOFTWARE_LICENSE_LABEL}`, 'repository')}
  {/if}
  {#if has('requests')}{@render row('Requests', t(requests), 'requests')}{/if}
  {@render row('Support it answers', form.demandNote, 'demand-note')}
  {@render row('Stages', stagesSummary(form.stages), 'stages')}
</div>
<p class="plan-rev-note">
  {t(
    "Members rate it on the plan criteria and the project's values. Your proposal counts as your Yes.",
  )}
</p>
