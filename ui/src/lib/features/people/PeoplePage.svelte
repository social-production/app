<script lang="ts">
  import LoadingLine from '#lib/components/loading/LoadingLine.svelte';
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import PersonChip from '#lib/components/PersonChip.svelte';
  import { explain } from '#lib/services/errors.ts';
  import { getPerson, listPeople, pairingOf } from '#lib/services/people.ts';

  import { personRows, type PersonRow } from './rows.ts';

  import '#lib/components/styles/profile.css';

  /** Home until projects land (M2c): everyone with a profile, one row per person. */
  let rows = $state<PersonRow[]>([]);
  let loaded = $state(false);
  let error = $state<string>();
  let query = $state('');
  const shown = $derived(
    query.trim()
      ? rows.filter((r) =>
          (r.person.profile?.name ?? '').toLowerCase().includes(query.trim().toLowerCase()),
        )
      : rows,
  );

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
    <header class="people-head">
      <h1>People</h1>
      <p>Open someone's profile to vouch for them.</p>
      <input
        id="people-search"
        class="people-search"
        type="search"
        placeholder="Search people"
        aria-label="Search people"
        bind:value={query}
      />
    </header>
    {#if error}<p class="warning-card">{error}</p>{/if}
    {#if !loaded}
      <LoadingLine /><SkeletonRows kind="row" count={6} />
    {:else}
      <div class="people-list people">
        {#each shown as row (row.person.agent)}
          <PersonChip person={row.person} devices={row.devices} />
        {:else}
          <p class="end-copy">
            {query.trim() ? `Nobody called "${query.trim()}".` : 'Nobody has set up a profile yet.'}
          </p>
        {/each}
      </div>
    {/if}
  </section>
</div>
