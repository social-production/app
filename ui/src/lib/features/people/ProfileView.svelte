<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import type { Snippet } from 'svelte';

  import PeopleSheet from '#lib/components/overlay/PeopleSheet.svelte';
  import ProfileHero from '#lib/components/people/ProfileHero.svelte';
  import EmptyState from '#lib/components/empty/EmptyState.svelte';
  import { acceptedCount, shownRows } from '#lib/domain/follows.ts';
  import { profileHidden } from '#lib/domain/privacy.ts';
  import { VOUCHES_FOR_FULL, type Person } from '#lib/domain/people.ts';
  import { explain } from '#lib/services/errors.ts';
  import { claimFullStanding, getPerson, pairingOf, vouchFor } from '#lib/services/people.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import BotButton from './BotButton.svelte';
  import BotReviewCard from './BotReviewCard.svelte';
  import EditProfileSheet from './EditProfileSheet.svelte';
  import FollowButton from './FollowButton.svelte';
  import FollowRequests from './FollowRequests.svelte';
  import { ProfileFollows } from './followState.svelte.ts';
  import MessageButton from './MessageButton.svelte';
  import ProfileShare from './ProfileShare.svelte';
  import { ProfileEditor } from './profileEdit.svelte.ts';
  import TrustSheets from './TrustSheets.svelte';
  import { ProfileTrust } from './trustState.svelte.ts';
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
  // ADR 0112: the person shows their activity to followers only.
  const hidden = $derived(!!follows.view && profileHidden(follows.view, agent, myAgent || null));
  const editor = $derived(new ProfileEditor(agent, load));
  const trust = $derived(new ProfileTrust(agent, load));
  // web: accounts found to be bots can't message, vouch or mark.
  const viewerPaused = $derived(me?.standing.level === 'Paused');
  const viewerFull = $derived(me?.standing.level === 'Full');
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
      await trust.load(p);
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
  $effect(() => {
    void editor.loadCover();
  });
</script>

<div class="profile-page">
  <section class="page">
    {#if person}
      {@const shown = person}
      <ProfileHero
        person={shown}
        {devices}
        trustOpen={trust.open}
        cover={editor.cover}
        onCoverFile={own ? (file) => void editor.pickCover(file) : undefined}
        onPhotoFile={own ? (file) => void editor.pickPhoto(shown, file) : undefined}
        onStanding={() => {
          trust.open = !trust.open;
          if (trust.open) follows.list = null;
        }}
      >
        {#snippet counts()}
          {@const view = follows.view}
          {#if view}
            {@const followers = acceptedCount(view.followers)}
            <button
              class:active={follows.list === 'followers'}
              type="button"
              onclick={() => {
                follows.list = follows.list === 'followers' ? null : 'followers';
                trust.open = false;
              }}
            >
              <strong>{followers}</strong>
              {followers === 1 ? 'follower' : 'followers'}
            </button>
            <button
              class:active={follows.list === 'following'}
              type="button"
              onclick={() => {
                follows.list = follows.list === 'following' ? null : 'following';
                trust.open = false;
              }}
            >
              <strong>{acceptedCount(view.following)}</strong>
              {t('following')}
            </button>
          {/if}
        {/snippet}
        {#snippet actions()}
          <!-- As on web, guests see no actions on someone's profile. -->
          {#if !own && me}
            {@const viewing = me}
            <div class="action-row">
              <FollowButton
                name={shown.profile?.name ?? 'them'}
                status={follows.view?.myStatus ?? null}
                busy={follows.busy || !follows.view}
                onToggle={() => void follows.toggle()}
              />
              {#if !viewerPaused}
                <MessageButton
                  person={{ agent: shown.agent, name: shown.profile?.name ?? 'Unnamed' }}
                />
                <ProfileShare agent={shown.agent} name={shown.profile?.name ?? 'Unnamed'} />
                <VouchButton
                  person={shown}
                  viewer={me}
                  {busy}
                  onVouch={() => run(() => vouchFor(agent))}
                />
                <BotButton
                  name={shown.profile?.name ?? 'them'}
                  marked={trust.markedBy(viewing.agent)}
                  canMark={viewerFull && !trust.review?.myCooldownUntil}
                  busy={trust.stancePending || !trust.review}
                  onToggle={() => trust.toggleMark(viewing.agent)}
                />
              {/if}
            </div>
          {/if}
        {/snippet}
        {#snippet aside()}
          <!-- Edit profile opens over the profile, beside the photo (Profiles mockups §2). -->
          {#if own}
            <button class="edit-profile" type="button" onclick={() => (editor.open = true)}
              >{t('Edit profile')}</button
            >
          {/if}
        {/snippet}
      </ProfileHero>
      {#if own}<EditProfileSheet {editor} person={shown} />{/if}
      {#if editor.error && !editor.open}
        <p class="warning-card" role="alert">{editor.error}</p>
      {/if}
      {#if follows.error}<p class="warning-card" role="alert">{follows.error}</p>{/if}
      {#if trust.stanceMessage}<p class="warning-card" role="alert">{trust.stanceMessage}</p>{/if}
      {#if trust.review && trust.review.status !== 'none'}
        <BotReviewCard
          review={trust.review}
          {own}
          canVote={!own && viewerFull && !viewerPaused}
          pending={trust.reviewPending}
          message={trust.reviewMessage}
          onVote={(vote) => void trust.vote(vote)}
          onReReview={() => void trust.reReview()}
        />
      {/if}
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
        title={follows.list === 'followers' ? t('Followers') : t('Following')}
        emptyCopy={follows.list === 'followers'
          ? t('No followers yet.')
          : t('Not following anyone yet.')}
        people={sheetPeople}
        onClose={() => (follows.list = null)}
      />
      <TrustSheets {trust} person={shown} />

      {#if own && canClaim}
        <section class="empty-card own-card">
          {#if canClaim}
            <div class="button-row">
              <button
                class="button-primary"
                type="button"
                disabled={busy}
                onclick={() => run(claimFullStanding)}
              >
                {t('Become a full member')}
              </button>
            </div>
          {/if}
        </section>
      {/if}
    {:else if !error}
      <SkeletonRows count={2} />
    {/if}
    {#if error}<p class="warning-card">{error}</p>{/if}
    {#if hidden}
      <EmptyState
        icon="shield"
        title={`${person?.profile?.name || 'This person'} shares their activity with followers`}
        text={t('Follow them to see their projects, threads, events and posts here.')}
      />
    {:else}
      {@render children?.()}
    {/if}
  </section>
</div>

<style>
  .own-card {
    display: grid;
    gap: 10px;
  }
</style>
