<script lang="ts">
  import { scheduleSummary } from '#lib/domain/eventSchedule.ts';
  import { SOFTWARE_LICENSE_LABEL } from '#lib/domain/policy.ts';
  import type { PlanForm, PlanStep, PlanStepType } from '#lib/domain/planForm.ts';

  /** web's plan wizard review step: every answer, each with an Edit link back to its step. */
  interface Props {
    form: PlanForm;
    steps: PlanStep[];
    onEdit: (step: PlanStepType) => void;
  }

  let { form, steps, onEdit }: Props = $props();

  const has = (id: PlanStepType): boolean => steps.some((s) => s.id === id);
  const placeLabel = $derived(has('distribution-location') ? 'Distribution location' : 'Location');
  const placeStep = $derived<PlanStepType>(
    has('distribution-location') ? 'distribution-location' : 'location',
  );
</script>

{#snippet row(label: string, value: string, step: PlanStepType)}
  <div class="review-row">
    <div class="review-copy">
      <strong>{label}</strong>
      <span>{value}</span>
    </div>
    <button class="text-button" type="button" onclick={() => onEdit(step)}>Edit</button>
  </div>
{/snippet}

<div class="review-stack">
  <div class="review-row">
    <div class="review-copy">
      <strong>Title & description</strong>
      <span><strong>{form.title}</strong> — {form.description}</span>
    </div>
    <button class="text-button" type="button" onclick={() => onEdit('plan-overview')}>Edit</button>
  </div>
  {#if has('schedule-mode')}
    {@render row('Schedule', scheduleSummary(form), 'schedule-mode')}
  {/if}
  {#if (has('location') || has('distribution-location')) && form.location}
    {@render row(placeLabel, form.location, placeStep)}
  {/if}
  {#if has('distribution-location') && form.recipients.trim()}
    {@render row('Recipients', form.recipients, 'distribution-location')}
  {/if}
  {#if has('repository') && form.repositoryUrl}
    {@render row('Repository', form.repositoryUrl, 'repository')}
    {@render row('License', SOFTWARE_LICENSE_LABEL, 'repository')}
  {/if}
  {@render row('Support response', form.demandNote, 'demand-note')}
  {#each form.stages as stage, index (index)}
    <div class="review-stage">
      {@render row(
        `Stage ${index + 1}: ${stage.title || 'Untitled'}`,
        stage.details || 'No details yet',
        'stages',
      )}
    </div>
  {/each}
</div>
