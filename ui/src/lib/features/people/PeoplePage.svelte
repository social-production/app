<script lang="ts">
  import PersonChip from '#lib/components/PersonChip.svelte';
  import { explain } from '#lib/services/errors.ts';
  import { getPerson, listPeople, pairingOf } from '#lib/services/people.ts';

  import { personRows, type PersonRow } from './rows.ts';

  import '#lib/components/styles/profile.css';

  /** Home until projects land (M2c): everyone with a profile, one row per person. */
  let rows = $state<PersonRow[]>([]);
  let loaded = $state(false);
  let error = $state<string>();

  async function load(): Promise<void> {
    try {
      const agents = await listPeople();
      const entries = await Promise.all(
        agents.map(async (agent) => ({
          person: await getPerson(agent),
          pairing: await pairingOf(agent),
        })),
      );
      rows = personRows(entries); // all at once, so the page doesn't jump while it fills in
    } catch (e) {
      error = explain(e);
    } finally {
      loaded = true;
    }
  }

  $effect(() => {
    void load();
  });
</script>

<div class="profile-page">
  <section class="page">
    <section class="hero-section">
      <h1>People</h1>
      <p class="profile-bio">Open someone's profile to vouch for them.</p>
    </section>
    {#if error}<p class="warning-card">{error}</p>{/if}
    {#if !loaded}
      <p class="end-copy">Loading people…</p>
    {:else}
      <div class="people-list people">
        {#each rows as row (row.person.agent)}
          <PersonChip person={row.person} devices={row.devices} />
        {:else}
          <p class="end-copy">Nobody has set up a profile yet.</p>
        {/each}
      </div>
    {/if}
  </section>
</div>

<style>
  .people {
    padding: 12px var(--page-gutter);
  }
</style>
