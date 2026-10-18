<script lang="ts">
  import CopyKey from '#lib/components/CopyKey.svelte';
  import ProfileHero from '#lib/components/people/ProfileHero.svelte';
  import { VOUCHES_FOR_FULL, type Person } from '#lib/domain/people.ts';
  import { explain } from '#lib/services/account.ts';
  import { claimFullStanding, getPerson, pairingOf, vouchFor } from '#lib/services/people.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import VouchButton from './VouchButton.svelte';

  import '#lib/components/styles/profile.css';

  /** web's profile page: hero with standing chip, and the Vouch action for others. */
  let { agent }: { agent: string } = $props();

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

  async function load(): Promise<void> {
    try {
      const [p, pairing, mine] = await Promise.all([
        getPerson(agent),
        pairingOf(agent),
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
</script>

<div class="profile-page">
  <section class="page">
    {#if person}
      {@const shown = person}
      <ProfileHero person={shown} {devices}>
        {#snippet actions()}
          {#if !own}
            <div class="action-row">
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
