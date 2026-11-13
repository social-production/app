<script lang="ts">
  import ContentMetaRow from '#lib/components/cards/ContentMetaRow.svelte';
  import FactLine from '#lib/components/cards/FactLine.svelte';
  import FeedCardTop from '#lib/components/cards/FeedCardTop.svelte';
  import ReportControl from '#lib/components/moderation/ReportControl.svelte';
  import RestrictionNotice from '#lib/components/moderation/RestrictionNotice.svelte';
  import { isHidden, type Moderation } from '#lib/domain/moderation.ts';
  import { zoneOf } from '#lib/domain/activityTime.ts';
  import { scheduleLine } from '#lib/domain/eventSchedule.ts';
  import type { PlanBoard } from '#lib/domain/plans.ts';
  import type { Charter, Project } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  /**
   * web's ProjectOverviewHeader context panel: the newest update is one line under the
   * description (web dropped its separate updates list; Update sits in the dock). An event
   * (web's EventOverviewHeader) shows its leading plan's time and place, as web-backend copies
   * them onto the event (`_sync_event_schedule_from_leading_plan`).
   */
  interface Props {
    project: Project;
    charter: Charter;
    plans: PlanBoard | undefined;
    names: Record<string, string>;
    /** web's ReportControl in the header (ADR 0071). */
    moderation: Moderation;
    onModeration: (next: Moderation) => void;
  }

  let { project, charter, plans, names, moderation, onModeration }: Props = $props();

  const leading = $derived(
    plans?.plans.find((p) => p.plan.kind === 'Event' && p.leader === 'Leading' && p.approved)?.plan,
  );
  const timeLabel = $derived(leading?.schedule ? scheduleLine(leading.schedule, zoneOf) : '');
  const place = $derived(leading?.location || charter.location_label);
  /** web strips a leading "Update:" so the line doesn't say it twice. */
  const latest = $derived(project.updates[0]?.body.replace(/^\s*update:\s*/i, '') ?? null);
</script>

<div class="project-overview">
  <div class="context-panel">
    <FeedCardTop
      mode={charter.mode}
      tags={charter.tags}
      stage={project.stage}
      isPrivate={project.audience !== 'public'}
    >
      <ReportControl
        id={project.id}
        itemLabel={charter.mode === 'event' ? 'event' : 'project'}
        {moderation}
        owner={project.creator}
        copyUrl={`${location.origin}${location.pathname}`}
        onChange={onModeration}
      />
    </FeedCardTop>
    <div class="heading overview-heading">
      <div class="identity-row">
        <div class="identity-copy">
          <RestrictionNotice active={isHidden(moderation)}>
            <h1>{charter.title}</h1>
          </RestrictionNotice>
          {#if timeLabel}<FactLine icon="when">{timeLabel}</FactLine>{/if}
          {#if place}<FactLine icon="where">{place}</FactLine>{/if}
          <p class="overview-copy">{charter.description}</p>
          {#if latest !== null}
            <p class="overview-update">Update: {latest}</p>
          {/if}
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
