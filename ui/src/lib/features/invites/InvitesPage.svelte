<script lang="ts">
  import LoadingLine from '#lib/components/loading/LoadingLine.svelte';
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import { endedAgo } from '#lib/domain/activityHistory.ts';
  import { zoneOf } from '#lib/domain/activityTime.ts';
  import { scheduleLine } from '#lib/domain/eventSchedule.ts';
  import { slugOf } from '#lib/domain/scopes.ts';
  import { explain } from '#lib/services/errors.ts';
  import EmptyState from '#lib/components/empty/EmptyState.svelte';
  import { answerInvite, pendingInvites, type PendingInvite } from '#lib/services/events.ts';
  import { notifications } from '#lib/stores/notifications.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import '#lib/components/styles/invites.css';

  /** web's InvitesPage: invites to private events and communities; nobody joins until they
   * accept (ADR 0061, 0068). */
  let invites = $state<PendingInvite[]>([]);
  let loading = $state(true);
  let error = $state('');
  let busyId = $state<string | null>(null);

  $effect(() => {
    pendingInvites()
      .then((found) => (invites = found))
      .catch((e: unknown) => (error = explain(e)))
      .finally(() => (loading = false));
  });

  async function answer(invite: PendingInvite, accept: boolean): Promise<void> {
    busyId = invite.seed;
    error = '';
    try {
      await answerInvite(invite.seed, accept);
      invites = invites.filter((i) => i.seed !== invite.seed);
      notifications.invites = invites.length;
      if (accept && invite.kind === 'community') {
        const page = resolve('/communities/[slug]', { slug: slugOf(invite.title) });
        // eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved above
        await goto(`${page}#invite=${invite.seed}`);
      } else if (accept) await goto(resolve('/events/[id]', { id: invite.seed }));
    } catch (e) {
      error = explain(e);
    } finally {
      busyId = null;
    }
  }
</script>

<section class="invites invites-ux">
  <header class="invites-head">
    <h1>Invites</h1>
    {#if !loading && invites.length > 0}
      <span>{invites.length} waiting for your answer</span>
    {/if}
  </header>
  {#if error}<p class="muted" role="alert">{error}</p>{/if}
  {#if loading}
    <LoadingLine /><SkeletonRows kind="row" count={3} />
  {:else if invites.length === 0}
    <EmptyState
      icon="people"
      title="No invites waiting"
      text="When someone invites you to a private community or event, it shows up here."
    />
  {:else}
    <ul>
      {#each invites as invite (invite.seed)}
        {@const when = invite.schedule ? scheduleLine(invite.schedule, zoneOf) : ''}
        <li>
          <div class="invite-head">
            <span class="invite-glyph" data-kind={invite.kind}>
              <SurfaceIcon icon={invite.kind === 'community' ? 'community' : 'event'} size="md" />
            </span>
            <span class="invite-who">
              <strong>{invite.title}</strong>
              <span class="kind"
                >{invite.kind === 'community'
                  ? 'Private community'
                  : 'Private event'}{#if invite.invitedBy}
                  · from {invite.invitedBy}{/if} · {endedAgo(invite.at)}</span
              >
            </span>
          </div>
          {#if invite.description}<p class="description">{invite.description}</p>{/if}
          {#if when}
            <p class="when">
              <svg aria-hidden="true" viewBox="0 0 24 24"
                ><rect x="4" y="5" width="16" height="15" rx="2" /><path
                  d="M4 9h16M8 3v4M16 3v4"
                /></svg
              >{when}
            </p>
          {/if}
          <div class="actions">
            <button
              class="decline"
              type="button"
              disabled={busyId === invite.seed}
              onclick={() => answer(invite, false)}>Decline</button
            >
            <button
              class="accept"
              type="button"
              disabled={busyId === invite.seed}
              onclick={() => answer(invite, true)}>Accept</button
            >
          </div>
        </li>
      {/each}
    </ul>
    <p class="muted">When someone invites you to a private community or event, it shows up here.</p>
  {/if}
</section>
