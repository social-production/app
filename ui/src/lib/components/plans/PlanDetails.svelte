<script lang="ts">
  import { SOFTWARE_LICENSE_LABEL } from '#lib/domain/policy.ts';
  import { SUBTYPE_LABELS, type Plan, type PlanStage } from '#lib/domain/plans.ts';

  /** web's plan sheet sections: Overview, Details, Notes and Stages. */
  let { plan }: { plan: Plan } = $props();

  const hasDetails = $derived(
    !!plan.subtype || !!plan.location || !!plan.recipients || !!plan.repository_url,
  );

  function day(value: string | null): string {
    if (!value) return '';
    const date = new Date(`${value}T00:00:00`);
    return Number.isNaN(date.getTime())
      ? value
      : date.toLocaleDateString(undefined, { day: 'numeric', month: 'short', year: 'numeric' });
  }

  function stageDates(stage: PlanStage): string {
    const [start, end] = [day(stage.start_date), day(stage.end_date)];
    return start && end ? `${start} – ${end}` : start || end;
  }

  /** web's `normalizeExternalUrl`. */
  function external(url: string): string {
    const trimmed = url.trim();
    return /^[a-zA-Z][a-zA-Z\d+.-]*:/.test(trimmed) ? trimmed : `https://${trimmed}`;
  }
</script>

<section class="plan-section">
  <h3>Overview</h3>
  <p class="plan-body">{plan.description}</p>
</section>

{#if hasDetails}
  <section class="plan-section">
    <h3>Details</h3>
    <dl class="plan-facts">
      {#if plan.subtype}
        <div class="fact">
          <dt>Subtype</dt>
          <dd>{SUBTYPE_LABELS[plan.subtype]}</dd>
        </div>
      {/if}
      {#if plan.location}
        <div class="fact">
          <dt>Location</dt>
          <dd>{plan.location}</dd>
        </div>
      {/if}
      {#if plan.subtype === 'software'}
        <div class="fact">
          <dt>Licence</dt>
          <dd>{SOFTWARE_LICENSE_LABEL}</dd>
        </div>
      {/if}
      {#if plan.recipients}
        <div class="fact">
          <dt>Recipients</dt>
          <dd>{plan.recipients}</dd>
        </div>
      {/if}
      {#if plan.repository_url}
        <div class="fact">
          <dt>Repository</dt>
          <dd>
            <a href={external(plan.repository_url)} rel="noreferrer" target="_blank"
              >{plan.repository_url}</a
            >
          </dd>
        </div>
      {/if}
    </dl>
  </section>
{/if}

<section class="plan-section">
  <h3>Notes</h3>
  <article class="note-card">
    <span class="note-label">Support response</span>
    <p>{plan.demand_note}</p>
  </article>
</section>

<section class="plan-section">
  <h3>Stages</h3>
  <ol class="stage-list">
    {#each plan.stages as stage, index (index)}
      <li class="stage-item">
        <span class="stage-num">{index + 1}</span>
        <div class="stage-copy">
          <strong>{stage.title}</strong>
          {#if stageDates(stage)}<span class="stage-meta">{stageDates(stage)}</span>{/if}
          <p>{stage.details}</p>
        </div>
      </li>
    {/each}
  </ol>
</section>
