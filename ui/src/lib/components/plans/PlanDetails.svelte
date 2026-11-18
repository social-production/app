<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { placeOnMap } from '#lib/components/map/placeOnMap.ts';
  import FactLine from '#lib/components/cards/FactLine.svelte';
  import { zoneOf } from '#lib/domain/activityTime.ts';
  import { scheduleLine } from '#lib/domain/eventSchedule.ts';
  import { SOFTWARE_LICENSE_LABEL } from '#lib/domain/policy.ts';
  import { SUBTYPE_LABELS, type Plan, type PlanStage } from '#lib/domain/plans.ts';
  import { REQUEST_MODES } from '#lib/domain/projects.ts';

  /** An opened plan read top to bottom like a page (Plan, Phase and Feedback Screens, 3):
   * what, when, where (opens the map), stages as numbered steps, its answer to support, and
   * the rest of its facts as a soft list. */
  let { plan, onMap }: { plan: Plan; onMap?: () => void } = $props();
  const map = placeOnMap();

  const place = $derived(plan.location?.trim() ?? '');
  const mappable = $derived(!!place && place.toLowerCase() !== 'online');
  const when = $derived(plan.schedule ? scheduleLine(plan.schedule, zoneOf) : '');
  const facts = $derived(
    [
      plan.subtype ? ['Type', SUBTYPE_LABELS[plan.subtype]] : null,
      plan.subtype === 'software' ? ['Licence', SOFTWARE_LICENSE_LABEL] : null,
      plan.requests
        ? ['Takes requests', REQUEST_MODES.find((m) => m.value === plan.requests)?.label ?? '']
        : null,
      plan.recipients ? ['Recipients', plan.recipients] : null,
    ].filter((f): f is [string, string] => f !== null),
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

  /** web's `normalizeExternalUrl`, http(s) only: plans come from other people, so a
   * `javascript:` repository must never become a live link. */
  function external(url: string): string {
    const trimmed = url.trim();
    return /^https?:\/\//i.test(trimmed) ? trimmed : `https://${trimmed}`;
  }

  function showPlace(e: MouseEvent): void {
    if (!map || e.metaKey || e.ctrlKey || e.shiftKey) return;
    e.preventDefault();
    onMap?.();
    map.show(place);
  }
</script>

<div class="plan-read">
  <p class="plan-body">{plan.description}</p>
  {#if when}<FactLine icon="when">{when}</FactLine>{/if}
  {#if mappable}
    <a
      class="plan-place"
      href={`https://www.openstreetmap.org/search?query=${encodeURIComponent(place)}`}
      rel="noopener noreferrer"
      onclick={showPlace}
    >
      <FactLine icon="where"><strong>{place}<small>{t('Show on the map')}</small></strong></FactLine
      >
      <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M9.5 6.5 15 12l-5.5 5.5" /></svg>
    </a>
  {:else if place}
    <FactLine icon="where">{place}</FactLine>
  {/if}

  {#if plan.stages.length > 0}
    <ol class="stage-list">
      {#each plan.stages as stage, index (index)}
        <li class="stage-item">
          <span class="stage-num">{index + 1}</span>
          <div class="stage-copy">
            <strong>{stage.title}</strong>
            {#if stageDates(stage)}<span class="stage-meta">{stageDates(stage)}</span>{/if}
            {#if stage.details}<p>{stage.details}</p>{/if}
          </div>
        </li>
      {/each}
    </ol>
  {/if}

  <div class="plan-note">
    <small>{t('How it answers support')}</small>
    {plan.demand_note}
  </div>

  {#if facts.length > 0 || plan.repository_url}
    <dl class="plan-facts">
      {#each facts as [label, value] (label)}
        <div class="fact">
          <dt>{label}</dt>
          <dd>{value}</dd>
        </div>
      {/each}
      {#if plan.repository_url}
        <div class="fact">
          <dt>{t('Repository')}</dt>
          <dd>
            <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- external link -->
            <a href={external(plan.repository_url)} rel="noreferrer" target="_blank"
              >{plan.repository_url}</a
            >
          </dd>
        </div>
      {/if}
    </dl>
  {/if}
</div>
