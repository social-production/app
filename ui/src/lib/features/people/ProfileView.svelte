<script lang="ts">
  import type { Snippet } from 'svelte';

  import CopyKey from '#lib/components/CopyKey.svelte';
  import PeopleSheet from '#lib/components/overlay/PeopleSheet.svelte';
  import ProfileHero from '#lib/components/people/ProfileHero.svelte';
  import { acceptedCount, shownRows } from '#lib/domain/follows.ts';
  import { VOUCHES_FOR_FULL, type Person } from '#lib/domain/people.ts';
  import { explain } from '#lib/services/errors.ts';
  import { claimFullStanding, getPerson, pairingOf, vouchFor } from '#lib/services/people.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import FollowButton from './FollowButton.svelte';
  import FollowRequests from './FollowRequests.svelte';
  import { ProfileFollows } from './followState.svelte.ts';
  import MessageButton from './MessageButton.svelte';
  import VouchButton from './VouchButton.svelte';

  import '#lib/components/styles/profile.css';

  /** web's profile page: hero with standing chip, and the Message and Vouch actions for others. */
  /** `children`: what the route shows under the header (the person's feed). */
  let { agent, children }: { agent: string; children?: Snippet | undefined } = $props();

  let person = $state<Person>();
  let me = $state<Person>();
  let devices = $state(1);
  let busy = $state(false);
  let error = $state<string>();

  const myAgent = $derived(
    connection.current.status === 'connected' ? connection.current.agent : '',
  );
  const own = $derived(agent === myAgent);
  const canClaim = $derived(
    own && person?.standing.level === 'New' && person.standing.vouchers.length >= VOUCHES_FOR_FULL,
  );

  const follows = $derived(new ProfileFollows(agent, own));
  const sheetPeople = $derived.by(() => {
    const view = follows.view;
    if (!view || !follows.list) return [];
    const rows = shownRows(follows.list === 'followers' ? view.followers : view.following, own);
    return rows.flatMap((r) => follows.people[r.agent] ?? []);
  });

  async function load(): Promise<void> {
    try {
      const [p, pairing, mine] = await Promise.all([
        getPerson(agent),
        // Guests can't read devices (holo/guest.ts allowlist); the hero then shows one.
        pairingOf(agent).catch(() => ({ primary: null, devices: [] })),
        myAgent ? getPerson(myAgent) : Promise.resolve(undefined),
      ]);
      person = p;
      me = mine;
      devices = 1 + pairing.devices.length;
    } catch (e) {
      error = explain(e);
    }
  }

  async function run(action: () => Promise<unknown>): Promise<void> {
    busy = true;
    error = undefined;
    try {
      await action();
      await load();
    } catch (e) {
      error = explain(e);
    } finally {
      busy = false;
    }
  }

  $effect(() => {
    void agent;
    void load();
  });
  $effect(() => {
    void follows.load();
  });
</script>

<div class="profile-page">
  <section class="page">
    {#if person}
      {@const shown = person}
      <ProfileHero person={shown} {devices}>
        {#snippet counts()}
          {@const view = follows.view}
          {#if view}
            {@const followers = acceptedCount(view.followers)}
            <button
              class:active={follows.list === 'followers'}
              type="button"
              onclick={() => (follows.list = follows.list === 'followers' ? null : 'followers')}
            >
              <strong>{followers}</strong>
              {followers === 1 ? 'follower' : 'followers'}
            </button>
            <button
              class:active={follows.list === 'following'}
              type="button"
              onclick={() => (follows.list = follows.list === 'following' ? null : 'following')}
            >
              <strong>{acceptedCount(view.following)}</strong> following
            </button>
          {/if}
        {/snippet}
        {#snippet actions()}
          <!-- As on web, guests see no actions on someone's profile. -->
          {#if !own && me}
            <div class="action-row">
              <FollowButton
                name={shown.profile?.name ?? 'them'}
                status={follows.view?.myStatus ?? null}
                busy={follows.busy || !follows.view}
                onToggle={() => void follows.toggle()}
              />
              {#if me && me.standing.level !== 'Paused'}
                <MessageButton
                  person={{ agent: shown.agent, name: shown.profile?.name ?? 'Unnamed' }}
                  viewer={me.agent}
                />
              {/if}
              <VouchButton
                person={shown}
                viewer={me}
                {busy}
                onVouch={() => run(() => vouchFor(agent))}
              />
            </div>
          {/if}
        {/snippet}
      </ProfileHero>
      {#if follows.error}<p class="warning-card" role="alert">{follows.error}</p>{/if}
      {#if own}
        <FollowRequests
          requests={follows.requests}
          people={follows.people}
          pending={follows.deciding}
          onDecide={(who, accept) => void follows.decide(who, accept)}
        />
      {/if}
      <PeopleSheet
        open={follows.list !== null}
        title={follows.list === 'followers' ? 'Followers' : 'Following'}
        emptyCopy={follows.list === 'followers' ? 'No followers yet.' : 'Not following anyone yet.'}
        people={sheetPeople}
        onClose={() => (follows.list = null)}
      />

      {#if own}
        <section class="empty-card own-card">
          {#if person.standing.level === 'New'}
            <p>
              New members can join projects, talk and help. To vote and vouch, ask
              {VOUCHES_FOR_FULL} full members to vouch for you ({person.standing.vouchers.length} so far),
              or stay active for 30 days.
            </p>
          {/if}
          {#if canClaim}
            <div class="button-row">
              <button
                class="button-primary"
                type="button"
                disabled={busy}
                onclick={() => run(claimFullStanding)}
              >
                Become a full member
              </button>
            </div>
          {/if}
          <CopyKey value={agent} />
        </section>
      {/if}
    {:else if !error}
      <p class="end-copy">Loading…</p>
    {/if}
    {#if error}<p class="warning-card">{error}</p>{/if}
    {@render children?.()}
  </section>
</div>

<style>
  .own-card {
    display: grid;
    gap: 10px;
  }
  .own-card p {
    margin: 0;
  }
</style>
