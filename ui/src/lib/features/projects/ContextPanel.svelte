<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import DetailFacts from '#lib/components/detail/DetailFacts.svelte';
  import FactRow from '#lib/components/detail/FactRow.svelte';
  import PlaceFact from '#lib/components/detail/PlaceFact.svelte';
  import FeedCardTop from '#lib/components/cards/FeedCardTop.svelte';
  import FoldedText from '#lib/components/cards/FoldedText.svelte';
  import ReportControl from '#lib/components/moderation/ReportControl.svelte';
  import RestrictionNotice from '#lib/components/moderation/RestrictionNotice.svelte';
  import { isHidden, type Moderation } from '#lib/domain/moderation.ts';
  import { zoneOf } from '#lib/domain/activityTime.ts';
  import { scheduleLine } from '#lib/domain/eventSchedule.ts';
  import type { PlanBoard } from '#lib/domain/plans.ts';
  import { namesLine } from '#lib/domain/people.ts';
  import type { Charter, Project } from '#lib/domain/projects.ts';
  import { relativeTime } from '#lib/domain/time.ts';

  import { resolve } from '$app/paths';

  /**
   * A project's or event's Details tab (Plan, Phase and Feedback Screens, 1): the channel
   * header, the title, who started it and when, the date (events), the place as a row that
   * opens the app's map, the description folded after about six lines (tags live only in the header, owner 17 Nov),
   * the latest update in its own soft block, and for an event the faces of who is going.
   * An event shows its leading plan's time and place, as web-backend copies them onto the
   * event (`_sync_event_schedule_from_leading_plan`).
   */
  interface Props {
    project: Project;
    charter: Charter;
    plans: PlanBoard | undefined;
    names: Record<string, string>;
    /** web's ReportControl in the header (ADR 0071). */
    moderation: Moderation;
    onModeration: (next: Moderation) => void;
    /** Opens the member list (the "N going" / "N members" row). */
    onMembers?: () => void;
    /** Opens Take part (the phase row). */
    onPhase?: () => void;
  }

  let { project, charter, plans, names, moderation, onModeration, onMembers, onPhase }: Props =
    $props();

  const event = $derived(charter.mode === 'event');
  const leading = $derived(
    plans?.plans.find((p) => p.plan.kind === 'Event' && p.leader === 'Leading' && p.approved)?.plan,
  );
  const timeLabel = $derived(leading?.schedule ? scheduleLine(leading.schedule, zoneOf) : '');
  const place = $derived((leading?.location || charter.location_label).trim());
  const update = $derived(project.updates[0] ?? null);
  /** web strips a leading "Update:" so the line doesn't say it twice. */
  const latest = $derived(update?.body.replace(/^\s*update:\s*/i, '') ?? null);
  const author = $derived(names[project.creator] ?? '…');
  const started = $derived(
    new Date(project.createdAt / 1000).toLocaleDateString(undefined, {
      day: 'numeric',
      month: 'short',
    }),
  );
  const going = $derived(project.members.slice(0, 3));
  const count = $derived(project.members.length);
  const peopleTitle = $derived(
    event ? `${count} going` : `${count} ${count === 1 ? 'member' : 'members'}`,
  );
  const peopleLine = $derived(
    namesLine(
      project.members.slice(0, 2).map((a) => names[a] ?? ''),
      count,
    ),
  );
  const phase = $derived(project.phases.find((p) => p.id === project.phase)?.label ?? '');
  const planCount = $derived(plans?.plans.length ?? 0);
  const phaseLine = $derived(
    planCount > 0 ? `${planCount} ${planCount === 1 ? 'plan' : 'plans'}` : '',
  );
</script>

{#snippet facts()}
  <DetailFacts>
    {#if project.lineage.continues}
      <FactRow
        icon="lineage"
        title={t('A converted project')}
        sub={t('Continues the work of')}
        href={resolve('/projects/[id]', { id: project.lineage.continues })}
      />
    {/if}
    {#if timeLabel}<FactRow icon="when" title={timeLabel} />{/if}
    <PlaceFact {place} />
    {#if count > 0}
      <FactRow
        icon="people"
        title={peopleTitle}
        sub={peopleLine}
        label={`${count} members`}
        onclick={() => onMembers?.()}
      >
        {#snippet aside()}
          {#each going as agent (agent)}<AvatarBadge size="sm" name={names[agent] ?? ''} />{/each}
        {/snippet}
      </FactRow>
    {/if}
    {#if !event && phase}
      <FactRow icon="phase" title={`${phase} phase`} sub={phaseLine} onclick={() => onPhase?.()} />
    {/if}
  </DetailFacts>
{/snippet}

<div class="project-overview">
  <div class="context-panel">
    <FeedCardTop
      mode={charter.mode}
      tags={charter.tags}
      stage={project.stage}
      isPrivate={project.audience !== 'public'}
      createdAt={project.createdAt}
    >
      <ReportControl
        id={project.id}
        itemLabel={event ? 'event' : 'project'}
        {moderation}
        owner={project.creator}
        copyUrl={`${location.origin}${location.pathname}`}
        onChange={onModeration}
      />
    </FeedCardTop>
    <div class="heading overview-heading">
      <RestrictionNotice active={isHidden(moderation)}>
        <h1>{charter.title}</h1>
      </RestrictionNotice>
      <p class="detail-by">
        <AvatarBadge size="sm" name={author} />
        <span>
          {event ? t('Organised by') : t('Started by')}
          <a href={resolve('/profile/[agent]', { agent: project.creator })}>{author}</a>
          · {started}
        </span>
      </p>
      {@render facts()}
      {#if charter.description.trim()}
        <FoldedText text={charter.description} className="overview-copy" />
      {/if}
      {#if update && latest !== null}
        <div class="overview-update">
          <small>Latest update · {relativeTime(update.at)}</small>
          {latest}
        </div>
      {/if}
    </div>
  </div>
</div>
