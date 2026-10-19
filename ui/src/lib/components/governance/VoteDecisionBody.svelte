<script lang="ts">
  import {
    APPROVAL_PERCENT,
    outcomeSentence,
    tallyLine,
    type Proposal,
  } from '#lib/domain/governance.ts';
  import type { Charter } from '#lib/domain/projects.ts';

  /** web's VoteDecisionBody: what passing changes, now vs proposed, and the tally. */
  interface Props {
    proposal: Proposal;
    authorName: string;
    charter: Charter;
    stage: string;
    phaseLabel: (id: string) => string;
  }

  let { proposal, authorName, charter, stage, phaseLabel }: Props = $props();

  const kind = $derived(proposal.kind);
</script>

<div class="vote-sheet">
  <p class="outcome">{outcomeSentence(kind, stage, phaseLabel)}</p>
  <p class="proposer">Proposed by {authorName}</p>

  {#if 'Update' in kind}
    <section class="change-card">
      <h3>Proposed update</h3>
      <p class="proposed-copy">{kind.Update.body}</p>
    </section>
  {:else if 'Edit' in kind}
    {#each [{ name: 'Title', now: charter.title, next: kind.Edit.title }, { name: 'Description', now: charter.description, next: kind.Edit.description }] as field (field.name)}
      {#if field.next !== null}
        <section class="change-card">
          <h3>{field.name}</h3>
          <div class="pair">
            <div>
              <p class="field-kicker">Now</p>
              <p>{field.now || '—'}</p>
            </div>
            <div class="proposed">
              <p class="field-kicker">Proposed</p>
              <p>{field.next}</p>
            </div>
          </div>
        </section>
      {/if}
    {/each}
    {#if kind.Edit.location !== null}
      <section class="change-card">
        <h3>Location</h3>
        <p class="proposed-copy">{kind.Edit.location || '—'}</p>
      </section>
    {/if}
  {/if}

  <section class="tally">
    <p>{tallyLine(proposal)}</p>
    <p class="tally-note">
      Needs {APPROVAL_PERCENT}% yes to pass, from at least {proposal.required} of {proposal.population}
      members.
    </p>
  </section>
  {#if proposal.myVote}<p class="clear-hint">
      You voted {proposal.myVote.toLowerCase()}. Votes are final.
    </p>{/if}
</div>
