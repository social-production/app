<script lang="ts">
  import ContentMetaRow from '#lib/components/cards/ContentMetaRow.svelte';
  import FeedCardTop from '#lib/components/cards/FeedCardTop.svelte';
  import UpdatesSection from '#lib/components/governance/UpdatesSection.svelte';
  import { zoneOf } from '#lib/domain/activityTime.ts';
  import { scheduleLine } from '#lib/domain/eventSchedule.ts';
  import type { PlanBoard } from '#lib/domain/plans.ts';
  import type { Charter, Project } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  /**
   * web's ProjectOverviewHeader context panel, with the updates section below it. An event
   * (web's EventOverviewHeader) shows its leading plan's time and place, as web-backend copies
   * them onto the event (`_sync_event_schedule_from_leading_plan`).
   */
  interface Props {
    project: Project;
    charter: Charter;
    plans: PlanBoard | undefined;
    names: Record<string, string>;
    canPropose: boolean;
    onAddUpdate: () => void;
  }

  let { project, charter, plans, names, canPropose, onAddUpdate }: Props = $props();

  const leading = $derived(
    plans?.plans.find((p) => p.plan.kind === 'Event' && p.leader === 'Leading' && p.approved)?.plan,
  );
  const timeLabel = $derived(leading?.schedule ? scheduleLine(leading.schedule, zoneOf) : '');
  const place = $derived(leading?.location || charter.location_label);
</script>

<div class="project-overview">
  <div class="context-panel">
    <FeedCardTop
      tags={charter.tags}
      stage={project.stage}
      isPrivate={project.audience !== 'public'}
    />
    <div class="heading overview-heading">
      <div class="identity-row">
        <div class="identity-copy">
          <h1>{charter.title}</h1>
          {#if timeLabel}<p class="live-fact">{timeLabel}</p>{/if}
          {#if place}<p class="live-fact">{place}</p>{/if}
          <p class="overview-copy">{charter.description}</p>
          {#if project.updates[0]}<p class="overview-update">
              Update: {project.updates[0].body}
            </p>{/if}
        </div>
      </div>
    </div>
    <div class="context-meta">
      <ContentMetaRow
        fullName
        authorName={names[project.creator] ?? '…'}
        authorHref={resolve('/profile/[agent]', { agent: project.creator })}
        createdAt={project.createdAt}
      />
    </div>
  </div>
</div>
<UpdatesSection updates={project.updates} {names} {canPropose} onAdd={onAddUpdate} />
