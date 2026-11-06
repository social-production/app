<script lang="ts">
  import type { Activity } from '#lib/domain/activities.ts';
  import {
    aggregate,
    cardTone,
    completionText,
    endedAgo,
    historyContext,
    needsFollowUp,
    owesCheckIn,
    ratingAverage,
    requesterSide,
    requesterText,
    staffingFailed,
  } from '#lib/domain/activityHistory.ts';

  import ActivityCard from './ActivityCard.svelte';
  import HistoryRating from './HistoryRating.svelte';

  /**
   * web's ActivityHistorySection: past rows with their completion badge and rating; the sheet
   * adds your follow-up (check-in, rating) and the community's. A booking (ADR 0077) adds the
   * requester's side. A personal service shows two sections with a heading each.
   */
  interface Props {
    items: Activity[];
    heading?: { title: string; description: string; empty: string } | null;
    names: Record<string, string>;
    me: string;
    planStageOf: (id: string | null) => string | null;
    onCheckIn: (activity: string, done: boolean) => void;
    onRate: (activity: string, stars: number | null, comment: string) => Promise<void>;
  }

  let p: Props = $props();
  let openComments = $state<Record<string, boolean>>({});

  const badgeTone = { complete: 'open', mixed: 'need', uncompleted: 'locked' } as const;
  const nameOf = (agent: string): string => p.names[agent] ?? 'Unnamed';
  const none = async (): Promise<{ agent: string; name: string }[]> => [];

  function summary(a: Activity): string {
    const average = ratingAverage(a);
    return average === null ? 'No ratings yet' : `${average}★`;
  }
</script>

{#snippet side(label: string, canSet: boolean, selection: boolean | null, id: string)}
  {#if canSet}
    <div class="completion-side-row">
      <span class="completion-side-label">{label}</span>
      <div class="completion-actions">
        <button
          class="vote-chip"
          class:selected={selection === true}
          type="button"
          onclick={() => selection !== true && p.onCheckIn(id, true)}>Completed</button
        >
        <button
          class="vote-chip negative"
          class:selected={selection === false}
          type="button"
          onclick={() => selection !== false && p.onCheckIn(id, false)}>Uncompleted</button
        >
      </div>
    </div>
  {/if}
{/snippet}

<section class="card-rail-section">
  {#if p.heading}
    <div class="section-head">
      <div class="section-copy">
        <h3>{p.heading.title}</h3>
        <p>{p.heading.description}</p>
      </div>
    </div>
    {#if p.items.length === 0}<div class="empty-card">{p.heading.empty}</div>{/if}
  {/if}
  <div class="card-rail history-rows">
    {#each [...p.items].sort((x, y) => y.endsAt - x.endsAt) as a (a.id)}
      {@const completion = aggregate(a)}
      {@const mine = a.ratings.find((r) => r.author === p.me) ?? null}
      {@const context = historyContext(a)}
      {@const requester = requesterSide(a)}
      <div id={`history-card-${a.id}`} class={`rail-card tone-${cardTone(a)}`}>
        <ActivityCard
          activity={a}
          names={p.names}
          me={p.me}
          planStage={p.planStageOf(a.planStage)}
          highlighted={false}
          readOnly
          canSuggest={false}
          onCommit={() => undefined}
          onSuggest={() => undefined}
          onDecline={() => undefined}
          searchPeople={none}
          history={{
            badge: completion.label,
            tone: badgeTone[completion.tone],
            rating: summary(a),
          }}
        >
          <div class="history-expanded-content">
            <div class="history-context">
              <p class="history-context-primary">
                <strong>{context.label}</strong>
                <span> — {context.description}</span>
              </p>
              <p class="history-context-meta">
                {#if a.booking}
                  <span>Requester: {nameOf(a.booking.requester)}</span>
                  <span class="meta-separator">·</span>
                {/if}
                <span>Ended {endedAgo(a.endsAt)}</span>
              </p>
            </div>
            <div class="history-expanded-actions">
              {#if needsFollowUp(a)}
                <section class="history-panel history-follow-up-panel">
                  <h4 class="history-section-heading">Your follow-up</h4>
                  {#if owesCheckIn(a)}
                    <div class="completion-follow-up">
                      {#if requester}
                        {@render side(
                          'Requester',
                          a.completion.requesterCanSet,
                          a.completion.requester,
                          a.id,
                        )}
                      {/if}
                      {@render side(
                        'Participants',
                        a.completion.canSet && !staffingFailed(a),
                        a.completion.mine,
                        a.id,
                      )}
                    </div>
                  {/if}
                  {#if a.canRate}
                    <HistoryRating
                      saved={mine}
                      onSave={(stars, comment) => p.onRate(a.id, stars, comment)}
                      onRemove={() => p.onRate(a.id, null, '')}
                    />
                  {/if}
                </section>
              {/if}
              <section class="history-panel history-community-panel">
                <h4 class="history-section-heading">Community</h4>
                <div class="community-completion-lines">
                  {#if requester}
                    <p><strong>Requester:</strong> {requesterText(requester)}</p>
                  {/if}
                  <p><strong>Participants:</strong> {completionText(a)}</p>
                </div>
                {#if a.ratings.length > 0}
                  <p class="community-rating-summary">
                    {ratingAverage(a)}★ · {a.ratings.length} ratings
                  </p>
                  <button
                    class="comments-toggle"
                    type="button"
                    onclick={() =>
                      (openComments = { ...openComments, [a.id]: !openComments[a.id] })}
                  >
                    {openComments[a.id] ? 'Hide comments' : `Show comments (${a.ratings.length})`}
                  </button>
                  {#if openComments[a.id]}
                    <div class="comments-list">
                      {#each a.ratings as rating (rating.author)}
                        <div class="comment-row">
                          <div class="comment-head">
                            <strong>{nameOf(rating.author)}</strong>
                            <span class="role-badge"
                              >{a.booking?.requester === rating.author
                                ? 'Requester'
                                : 'Participant'}</span
                            >
                            <span>{rating.stars}★</span>
                          </div>
                          {#if rating.comment}<p>{rating.comment}</p>{/if}
                        </div>
                      {/each}
                    </div>
                  {/if}
                {/if}
              </section>
            </div>
          </div>
        </ActivityCard>
      </div>
    {/each}
  </div>
</section>
