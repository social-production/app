<script lang="ts">
  import { t, plural } from '#lib/i18n/index.ts';
  import { resultSentence } from '#lib/domain/decided.ts';
  import { APPROVAL_PERCENT, type Proposal } from '#lib/domain/governance.ts';
  import { relativeTime } from '#lib/domain/time.ts';
  import { isNetworkVote } from '#lib/domain/board.ts';
  import type { Charter } from '#lib/domain/projects.ts';
  import { settingsSummary } from '#lib/domain/serviceRequests.ts';

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
  const total = $derived(proposal.yes + proposal.no);
  const percent = $derived(total > 0 ? Math.round((proposal.yes / total) * 100) : 0);
  const turnout = $derived(
    `${total} of ${plural(proposal.population, '{n} member', '{n} members')} voted · at least ${proposal.required} needed`,
  );
  const state = $derived(
    proposal.status === 'Open' ? 'open' : proposal.status === 'Passed' ? 'passed' : 'failed',
  );
</script>

<div class="vote-sheet">
  <!-- History's sheets (owner, 2026-11-18), drawn as the Channel Pictures vote: the result as a
       chip, what it does or did, then one bar towards the 66% mark. -->
  <div class="vote-status-row">
    <span class={`vote-status ${state}`}
      >{state === 'open' ? t('Open') : state === 'passed' ? t('Passed') : t("Didn't pass")}</span
    >
    <span class="proposer">{authorName} · {relativeTime(proposal.createdAt)}</span>
  </div>
  <p class="outcome">{resultSentence(proposal, stage, phaseLabel)}</p>
  {#if isNetworkVote(charter.tags)}
    <p class="network-vote-line">{t('Network vote: any Full member can vote')}</p>
  {/if}

  {#if 'Update' in kind}
    <section class="change-card">
      <h3>{t('Proposed update')}</h3>
      <p class="proposed-copy">{kind.Update.body}</p>
    </section>
  {:else if 'Return' in kind || 'Phase' in kind || 'Convert' in kind}
    <!-- web shows a phase change's reason as "Why". -->
    <section class="change-card">
      <h3>{t('Why')}</h3>
      <p>
        {'Return' in kind
          ? kind.Return.reason
          : 'Convert' in kind
            ? kind.Convert.reason
            : kind.Phase.reason}
      </p>
    </section>
  {:else if 'Software' in kind}
    <!-- web's generic decision: the change's card, then why (software votes, ADR 0086). -->
    {#if kind.Software.title && kind.Software.title !== kind.Software.label}
      <section class="change-card">
        <h3>{kind.Software.title}</h3>
        {#if kind.Software.description}<p>{kind.Software.description}</p>{/if}
      </section>
    {/if}
    {#if kind.Software.reason}
      <section class="change-card">
        <h3>{t('Why')}</h3>
        <p>{kind.Software.reason}</p>
      </section>
    {/if}
  {:else if 'RequestSettings' in kind}
    <!-- web's request_settings vote: the proposed setup, then why (ADR 0089). -->
    <section class="change-card">
      <h3>
        {settingsSummary({
          enabled: kind.RequestSettings.enabled,
          mode: kind.RequestSettings.mode,
          offSchedule: kind.RequestSettings.off_schedule,
        })}
      </h3>
    </section>
    {#if kind.RequestSettings.reason}
      <section class="change-card">
        <h3>{t('Why')}</h3>
        <p>{kind.RequestSettings.reason}</p>
      </section>
    {/if}
  {:else if 'Edit' in kind}
    {#each [{ name: 'Title', now: charter.title, next: kind.Edit.title }, { name: 'Description', now: charter.description, next: kind.Edit.description }] as field (field.name)}
      {#if field.next !== null}
        <section class="change-card">
          <h3>{field.name}</h3>
          <div class="pair">
            <div>
              <p class="field-kicker">{t('Now')}</p>
              <p>{field.now || '—'}</p>
            </div>
            <div class="proposed">
              <p class="field-kicker">{t('Proposed')}</p>
              <p>{field.next}</p>
            </div>
          </div>
        </section>
      {/if}
    {/each}
    {#if kind.Edit.location !== null}
      <section class="change-card">
        <h3>{t('Location')}</h3>
        <p class="proposed-copy">{kind.Edit.location || '—'}</p>
      </section>
    {/if}
  {/if}

  <section class="tally">
    <div class="tally-line">
      <b>{total === 0 ? t('No votes yet') : `${proposal.yes} yes · ${proposal.no} no`}</b>
      <span>{total > 0 ? `${percent}% yes · ` : ''}needs {APPROVAL_PERCENT}%</span>
    </div>
    <div class="tally-bar" class:failed={state === 'failed'}>
      <i style={`width: ${percent}%`}></i>
      <span class="tally-mark" style={`left: ${APPROVAL_PERCENT}%`}></span>
    </div>
    <p class="tally-note">{turnout}</p>
    <!-- App-only (ADR 0040): challenged counts and members still left out. -->
    {#if proposal.added > 0}
      <p class="tally-note">
        Reopened: a challenge added {plural(proposal.added, '{n} member', '{n} members')} the first count
        left out.
      </p>
    {/if}
    {#if proposal.canChallenge}
      <p class="tally-note">
        {proposal.missing}
        {proposal.missing === 1 ? t('member who joined') : t('members who joined')} before this was proposed
        {proposal.missing === 1 ? "isn't" : "aren't"} counted. Challenge the count in the Participation
        tab.
      </p>
    {/if}
  </section>
  {#if proposal.myVote}<p class="clear-hint">
      You voted {proposal.myVote.toLowerCase()}. Votes are final.
    </p>{/if}
</div>
