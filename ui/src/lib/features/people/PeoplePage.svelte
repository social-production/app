<script lang="ts">
  import PersonChip from '#lib/components/PersonChip.svelte';
  import type { Person } from '#lib/domain/people.ts';
  import PasskeyOffer from '#lib/features/account/PasskeyOffer.svelte';
  import { explain } from '#lib/services/account.ts';
  import { getPerson, listPeople, pairingOf, vouchFor } from '#lib/services/people.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { personRows, type PersonRow } from './rows.ts';
  import StandingPanel from './StandingPanel.svelte';

  let rows = $state<PersonRow[]>([]);
  let me = $state<Person>();
  let loaded = $state(false);
  let error = $state<string>();
  let busy = $state<string>();

  const myAgent = $derived(
    connection.current.status === 'connected' ? connection.current.agent : '',
  );
  const canVouch = $derived(me?.standing.level === 'Full');

  async function load(): Promise<void> {
    try {
      const agents = await listPeople();
      const [entries, mine] = await Promise.all([
        Promise.all(
          agents.map(async (agent) => ({
            person: await getPerson(agent),
            pairing: await pairingOf(agent),
          })),
        ),
        myAgent ? getPerson(myAgent) : Promise.resolve(undefined),
      ]);
      // Set everything at once so the page doesn't jump while it fills in.
      rows = personRows(entries);
      me = mine;
    } catch (e) {
      error = explain(e);
    } finally {
      loaded = true;
    }
  }

  function vouchable(person: Person): boolean {
    return (
      canVouch &&
      person.agent !== myAgent &&
      person.standing.level === 'New' &&
      !person.standing.vouchers.includes(myAgent)
    );
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

{#if !loaded}
  <p class="muted">Loading people…</p>
{:else}
  <div class="stack">
    {#if me}
      <PasskeyOffer name={me.profile?.name ?? 'Social Production'} />
      <StandingPanel person={me} onChange={load} />
    {/if}
    <section class="panel stack">
      <h2>People</h2>
      {#if error}<p class="error">{error}</p>{/if}
      {#each rows as row (row.person.agent)}
        <PersonChip person={row.person} devices={row.devices}>
          {#snippet actions()}
            {#if vouchable(row.person)}
              <button onclick={() => vouch(row.person)} disabled={busy === row.person.agent}
                >Vouch</button
              >
            {/if}
          {/snippet}
        </PersonChip>
      {:else}
        <p class="muted">Nobody has set up a profile yet.</p>
      {/each}
    </section>
  </div>
{/if}
