<script lang="ts">
  import PlanWizardShell from '#lib/components/plans/PlanWizardShell.svelte';
  import {
    activitySteps,
    localNow,
    minimumPeople,
    namedRoles,
    placeSummary,
    scheduleProblem,
    stepReady,
    type ActivityForm,
  } from '#lib/domain/activityForm.ts';

  import RolesEditor from './RolesEditor.svelte';

  import '#lib/components/styles/activities.css';

  /**
   * web's ActivityCreationWizard: title, schedule, place (a plain label or Online until places
   * land), the plan stage when the leading plans have stages, roles, note, review.
   */
  interface Props {
    open: boolean;
    form: ActivityForm;
    planStages: { id: string; label: string; group: string }[];
    busy: boolean;
    error: string;
    onSubmit: () => void;
    onCancel: () => void;
    searchPeople: (q: string) => Promise<{ agent: string; name: string }[]>;
  }

  let {
    open,
    form = $bindable(),
    planStages,
    busy,
    error,
    onSubmit,
    onCancel,
    searchPeople,
  }: Props = $props();

  let index = $state(0);
  const steps = $derived(activitySteps(planStages.length > 0));
  const step = $derived(steps[index] ?? steps[0]);
  const review = $derived(step?.type === 'review');
  const problem = $derived(step?.type === 'schedule' ? scheduleProblem(form) : null);
  const groups = $derived(
    planStages.reduce<{ label: string; stages: typeof planStages }[]>((out, s) => {
      const group = out.find((g) => g.label === s.group);
      if (group) group.stages.push(s);
      else out.push({ label: s.group, stages: [s] });
      return out;
    }, []),
  );
  const stageLabel = $derived(planStages.find((s) => s.id === form.planStage)?.label ?? '—');

  function close(): void {
    index = 0;
    onCancel();
  }

  function next(): void {
    if (review) onSubmit();
    else index += 1;
  }

  const when = (value: string): string =>
    value
      ? new Date(value).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' })
      : '—';
</script>

<PlanWizardShell
  {open}
  title="Create activity"
  stepIndex={index}
  stepCount={steps.length}
  nextLabel={review ? (busy ? 'Creating…' : 'Create activity') : 'Next'}
  canGoBack={index > 0}
  canGoNext={!busy && !!step && stepReady(step.type, form)}
  onClose={close}
  onDismiss={onCancel}
  onBack={() => (index = Math.max(0, index - 1))}
  onNext={next}
>
  {#if step}
    <div class="activity-wizard">
      <div class="question-block">
        <h2>{step.question}</h2>
        <p class="helper-copy">{step.helper}</p>
        {#if step.type === 'title'}
          <input bind:value={form.title} maxlength="120" placeholder="Activity title" />
        {:else if step.type === 'schedule'}
          <div class="field-grid">
            <label>
              <span>Start time</span>
              <input bind:value={form.scheduledAt} type="datetime-local" min={localNow()} />
            </label>
            <label>
              <span>Finish time</span>
              <input
                bind:value={form.endsAt}
                type="datetime-local"
                min={form.scheduledAt || localNow()}
              />
            </label>
          </div>
          {#if problem}<p class="validation-copy">{problem}</p>{/if}
        {:else if step.type === 'location'}
          <label class="online-toggle">
            <input bind:checked={form.online} type="checkbox" />
            <span>Online</span>
          </label>
          <input
            bind:value={form.place}
            aria-label={form.online ? 'Online detail' : 'Place'}
            maxlength="160"
            placeholder={form.online ? 'Online detail (optional)' : 'Place or address'}
          />
          {#if form.online}
            <label>
              <span>Meeting link</span>
              <input
                bind:value={form.meetingUrl}
                type="url"
                inputmode="url"
                maxlength="500"
                placeholder="https://"
              />
            </label>
            <p class="helper-copy">Members use this link to join.</p>
          {/if}
        {:else if step.type === 'plan-stage'}
          <select bind:value={form.planStage}>
            <option value={null} disabled>Choose stage</option>
            {#each groups as group (group.label)}
              <optgroup label={group.label}>
                {#each group.stages as stage (stage.id)}
                  <option value={stage.id}>{stage.label}</option>
                {/each}
              </optgroup>
            {/each}
          </select>
        {:else if step.type === 'roles'}
          <RolesEditor bind:roles={form.roles} {searchPeople} />
          <div class="count-readout">
            <span>Minimum people needed:</span>
            <strong>{minimumPeople(form.roles)}</strong>
          </div>
        {:else if step.type === 'note'}
          <textarea
            bind:value={form.note}
            rows="5"
            placeholder="What should happen in this activity?"></textarea>
        {:else}
          <div class="review-stack">
            <div class="review-row"><span>Title</span><strong>{form.title || '—'}</strong></div>
            <div class="review-row">
              <span>Schedule</span><strong>{when(form.scheduledAt)} – {when(form.endsAt)}</strong>
            </div>
            <div class="review-row">
              <span>Location</span><strong>{placeSummary(form) || '—'}</strong>
            </div>
            {#if planStages.length > 0}
              <div class="review-row"><span>Linked stage</span><strong>{stageLabel}</strong></div>
            {/if}
            <div class="review-row">
              <span>Roles</span>
              <strong>
                {namedRoles(form.roles)
                  .map((r) => `${r.label} (${r.required}${r.maximum ? `–${r.maximum}` : '+'})`)
                  .join(', ') || '—'}
              </strong>
            </div>
            <div class="review-row"><span>Note</span><strong>{form.note || '—'}</strong></div>
          </div>
          {#if error}<p class="validation-copy" role="alert">{error}</p>{/if}
        {/if}
      </div>
    </div>
  {/if}
</PlanWizardShell>
