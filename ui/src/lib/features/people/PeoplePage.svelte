<script lang="ts">
  import PersonChip from '#lib/components/PersonChip.svelte';
  import type { Person } from '#lib/domain/people.ts';
  import { explain } from '#lib/services/account.ts';
  import { getPerson, listPeople, vouchFor } from '#lib/services/people.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import PasskeyOffer from '#lib/features/account/PasskeyOffer.svelte';

  import StandingPanel from './StandingPanel.svelte';

  let people = $state<Person[]>([]);
  let me = $state<Person>();
  let error = $state<string>();
  let busy = $state<string>();

  const myAgent = $derived(
    connection.current.status === 'connected' ? connection.current.agent : '',
  );
  const canVouch = $derived(me?.standing.level === 'Full');

  async function load(): Promise<void> {
    try {
      const agents = await listPeople();
      people = await Promise.all(agents.map(getPerson));
      me = myAgent ? await getPerson(myAgent) : undefined;
    } catch (e) {
      error = explain(e);
    }
  }

  async function vouch(person: Person): Promise<void> {
    busy = person.agent;
    error = undefined;
    try {
      await vouchFor(person.agent);
      await load();
    } catch (e) {
      error = explain(e);
    } finally {
      busy = undefined;
    }
  }

  $effect(() => {
    void load();
  });
</script>

<div class="stack">
  {#if me}<PasskeyOffer name={me.profile?.name ?? 'Social Production'} />{/if}
  {#if me}<StandingPanel person={me} onChange={load} />{/if}
  <section class="panel stack">
    <h2>People</h2>
    {#if error}<p class="error">{error}</p>{/if}
    {#each people as person (person.agent)}
      <PersonChip {person}>
        {#snippet actions()}
          {#if canVouch && person.agent !== myAgent && !person.standing.vouchers.includes(myAgent) && person.standing.level === 'New'}
            <button onclick={() => vouch(person)} disabled={busy === person.agent}>Vouch</button>
          {/if}
        {/snippet}
      </PersonChip>
    {:else}
      <p class="muted">Nobody here yet. Set up your profile to appear in this list.</p>
    {/each}
  </section>
</div>
