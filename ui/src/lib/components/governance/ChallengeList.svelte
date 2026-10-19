<script lang="ts">
  import { proposalLabel, type Proposal } from '#lib/domain/governance.ts';

  /**
   * App-only (web has no challenge window; docs/GOVERNANCE.md, ADR 0040): decisions whose count
   * left out members who had joined before them, while a challenge still counts.
   */
  interface Props {
    proposals: Proposal[];
    phaseLabel: (id: string) => string;
    pending: boolean;
    onChallenge: (proposal: Proposal) => void;
  }

  let { proposals, phaseLabel, pending, onChallenge }: Props = $props();

  const when = (micros: number): string =>
    new Date(micros / 1000).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });

  function summary(p: Proposal): string {
    const people =
      p.missing === 1
        ? 'member who had joined before it is'
        : 'members who had joined before it are';
    return `${p.status}. It counts ${p.population} members; ${p.missing} ${people} left out.`;
  }
</script>

{#if proposals.length > 0}
  <section class="challenge-list" aria-labelledby="challenge-list-title">
    <h3 id="challenge-list-title">Counts you can challenge</h3>
    <p>
      A challenge adds the missing members to the count. The decision stops applying and reopens
      until enough of the larger group votes.
    </p>
    {#each proposals as p (p.id)}
      <article class="challenge-card">
        <strong>{proposalLabel(p.kind, phaseLabel)}</strong>
        <p>{summary(p)}</p>
        {#if p.challengeUntil}<p>Challenges count until {when(p.challengeUntil)}.</p>{/if}
        <button
          class="button-secondary"
          type="button"
          disabled={pending}
          onclick={() => onChallenge(p)}>Challenge the count</button
        >
      </article>
    {/each}
  </section>
{/if}
