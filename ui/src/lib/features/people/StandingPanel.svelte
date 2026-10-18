<script lang="ts">
  import CopyKey from '#lib/components/CopyKey.svelte';
  import PersonChip from '#lib/components/PersonChip.svelte';
  import { VOUCHES_FOR_FULL, type Person } from '#lib/domain/people.ts';
  import { explain } from '#lib/services/account.ts';
  import { claimFullStanding } from '#lib/services/people.ts';

  let { person, onChange }: { person: Person; onChange: () => void } = $props();

  let busy = $state(false);
  let error = $state<string>();

  const ready = $derived(
    person.standing.level === 'New' && person.standing.vouchers.length >= VOUCHES_FOR_FULL,
  );

  async function claim(): Promise<void> {
    busy = true;
    error = undefined;
    try {
      await claimFullStanding();
      onChange();
    } catch (e) {
      error = explain(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="panel stack">
  <h2>You</h2>
  <PersonChip {person} />
  {#if person.standing.level === 'New'}
    <p class="muted">
      New members can join projects, talk and help. To vote and vouch, ask
      {VOUCHES_FOR_FULL} full members to vouch for you ({person.standing.vouchers.length} so far), or
      stay active for 30 days.
    </p>
    {#if ready}
      <button class="primary" onclick={claim} disabled={busy}>Become a full member</button>
    {/if}
  {/if}
  {#if error}<p class="error">{error}</p>{/if}
  <CopyKey value={person.agent} />
</section>
