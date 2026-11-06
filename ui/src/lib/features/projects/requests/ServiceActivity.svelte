<script lang="ts">
  import ActivityAgenda from '#lib/components/activities/ActivityAgenda.svelte';
  import type { Activity } from '#lib/domain/activities.ts';
  import { ruleLine, type AvailabilityRule } from '#lib/domain/availability.ts';
  import type { Chip } from '#lib/domain/serviceAgenda.ts';

  import type { ActivitiesState } from '../activitiesState.svelte.ts';

  /**
   * web's personal-service "Activity" section (ADR 0077): the counts line, the creator's weekly
   * hours with Remove, and the agenda: the creator's activities, or everyone else's slots,
   * which a tap requests.
   */
  interface Props {
    activities: ActivitiesState;
    creator: boolean;
    me: string;
    member: boolean;
    usesCalendar: boolean;
    counts: string;
    /** Nothing open or accepted ahead (web's empty-card condition). */
    nothingAhead: boolean;
    rules: AvailabilityRule[];
    rows: Activity[];
    chipOf: (a: Activity) => Chip;
    onRemoveRule: (id: string) => void;
    onSelect: ((id: string) => void) | null;
  }

  let p: Props = $props();
</script>

<section class="card-rail-section service-ui">
  <div class="section-head">
    <div class="section-copy">
      <h3>Activity</h3>
      <p>{p.counts}</p>
    </div>
  </div>

  {#if p.creator && p.rules.length > 0}
    <div class="rules-list">
      {#each p.rules as rule (rule.id)}
        <div class="rule-row">
          <span>{ruleLine(rule)}</span>
          {#if rule.note}<span class="field-help">{rule.note}</span>{/if}
          <button class="text-button" type="button" onclick={() => p.onRemoveRule(rule.id)}
            >Remove</button
          >
        </div>
      {/each}
    </div>
  {/if}

  {#if !p.activities.loaded}
    <div class="empty-card">Loading activities…</div>
  {:else if p.rows.length === 0}
    {#if !p.usesCalendar}
      <div class="empty-card">This service is currently running without listed activity slots.</div>
    {:else if p.nothingAhead}
      <div class="empty-card">
        No future availability or accepted request activity is scheduled yet.
      </div>
    {/if}
  {:else}
    <ActivityAgenda
      activities={p.rows}
      names={p.activities.names}
      me={p.me}
      planStageOf={() => null}
      highlighted={p.activities.highlighted}
      readOnly={!p.creator}
      canSuggest={p.creator && p.member}
      onCommit={p.activities.commit}
      onSuggest={p.activities.suggest}
      onDecline={p.activities.decline}
      searchPeople={p.activities.searchPeople}
      chipOf={p.chipOf}
      onSelect={p.onSelect}
    />
  {/if}
</section>
