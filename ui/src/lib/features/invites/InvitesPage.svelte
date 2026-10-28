<script lang="ts">
  import { endedAgo } from '#lib/domain/activityHistory.ts';
  import { zoneOf } from '#lib/domain/activityTime.ts';
  import { scheduleLine } from '#lib/domain/eventSchedule.ts';
  import { explain } from '#lib/services/errors.ts';
  import { answerInvite, pendingInvites, type PendingInvite } from '#lib/services/events.ts';
  import { notifications } from '#lib/stores/notifications.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import '#lib/components/styles/invites.css';

  /** web's InvitesPage: invites to private events; nobody joins until they accept (ADR 0061). */
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
      if (accept) await goto(resolve('/events/[id]', { id: invite.seed }));
    } catch (e) {
      error = explain(e);
    } finally {
      busyId = null;
    }
  }
</script>

<section class="invites">
  <h1>Invites</h1>
  {#if error}<p class="muted" role="alert">{error}</p>{/if}
  {#if loading}
    <p class="muted">Loading invites…</p>
  {:else if invites.length === 0}
    <p class="muted">
      No invites waiting. When someone invites you to a private community or event, it shows up
      here.
    </p>
  {:else}
    <ul>
      {#each invites as invite (invite.seed)}
        {@const when = invite.schedule ? scheduleLine(invite.schedule, zoneOf) : ''}
        <li>
          <p class="kind">
            Private event{#if invite.invitedBy}
              · from {invite.invitedBy}{/if} · {endedAgo(invite.at)}
          </p>
          <strong>{invite.title}</strong>
          {#if when}<span class="when">{when}</span>{/if}
          {#if invite.description}<p class="description">{invite.description}</p>{/if}
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
  {/if}
</section>
