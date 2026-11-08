<script lang="ts">
  import PlanWizardShell from '#lib/components/plans/PlanWizardShell.svelte';

  import PullRequestReview from './PullRequestReview.svelte';
  import type { SoftwareState } from './softwareState.svelte.ts';
  import {
    emptyForms,
    request,
    stepCount,
    submitLabel,
    validation,
    wizardTitle,
  } from './softwareWizard.ts';

  import '#lib/components/styles/software.css';

  /** web's SoftwareGovernanceWizard (ADR 0086): five modes in the plan wizard's shell. */
  let { sw }: { sw: SoftwareState } = $props();

  const g = $derived(sw.governance);
  const mode = $derived(sw.mode);
  const pr = $derived(sw.active);
  const replaceable = $derived(g?.pullRequests.filter((p) => p.stage === 'awaiting-merge') ?? []);
  const name = (agent: string): string => sw.names[agent] || 'unknown';

  let step = $state(0);
  let forms = $state(emptyForms());

  // A fresh form each time the wizard opens, seeded as web seeds it.
  $effect(() => {
    if (mode) {
      step = 0;
      forms = emptyForms(g?.candidates[0] ?? '', replaceable[0]?.id ?? '');
    }
  });

  const count = $derived(stepCount(mode));
  const message = $derived(validation(mode, step, forms));
  const canGoNext = $derived(!sw.busy && message === null && (mode !== 'record-merge' || !!pr));

  async function next(): Promise<void> {
    if (step < count - 1) {
      step += 1;
      return;
    }
    const filed = request(mode, forms, pr);
    if (!filed) {
      sw.close();
      return;
    }
    if (await sw.submit(filed)) sw.close();
  }

  async function vote(choice: 'Yes' | 'No'): Promise<void> {
    if (pr && (await sw.vote(pr, choice))) sw.close();
  }
</script>

{#if mode && g}
  <PlanWizardShell
    open={true}
    title={wizardTitle(mode, pr)}
    stepIndex={step}
    stepCount={count}
    nextLabel={step >= count - 1 ? submitLabel(mode) : 'Next'}
    canGoBack={step > 0}
    {canGoNext}
    showFooter={mode !== 'vote-pr'}
    onClose={() => sw.close()}
    onDismiss={() => sw.close()}
    onBack={() => (step = Math.max(step - 1, 0))}
    onNext={() => void next()}
  >
    <div class="software-wizard">
      <div class="question-block">
        {#if mode === 'create-pr'}
          {#if step === 0}
            <h2>What should we call this pull request?</h2>
            <p class="helper-copy">
              Use a short title that matches the change people will vote on.
            </p>
            <input bind:value={forms.title} maxlength="120" placeholder="Pull request title" />
          {:else if step === 1}
            <h2>Where is the pull request?</h2>
            <p class="helper-copy">Link the external PR so voters can inspect the change.</p>
            <div class="field-grid">
              <input
                bind:value={forms.pullRequestId}
                maxlength="40"
                placeholder="PR number or ID"
              />
              <input
                bind:value={forms.pullRequestUrl}
                maxlength="240"
                placeholder="Pull request URL"
              />
            </div>
          {:else if step === 2}
            <h2>What does this change?</h2>
            <p class="helper-copy">
              Summarise the effect in plain language for non-specialist voters.
            </p>
            <textarea
              bind:value={forms.summary}
              rows="5"
              placeholder="What does this pull request change?"></textarea>
          {:else}
            <h2>Ready to submit?</h2>
            <div class="review-card">
              <strong>{forms.title}</strong>
              <span>{forms.pullRequestId} · {forms.pullRequestUrl}</span>
              <p>{forms.summary}</p>
            </div>
          {/if}
        {:else if (mode === 'vote-pr' || mode === 'record-merge') && pr}
          <PullRequestReview {pr} {mode} author={name(pr.author)} onVote={(c) => void vote(c)}>
            <div class="field-stack">
              <label class="field-label" for="merge-id-input">Merge ID</label>
              <input
                id="merge-id-input"
                bind:value={forms.mergeId}
                maxlength="120"
                placeholder="Merge commit or release ID"
              />
              <label class="field-label" for="merge-url-input">Merge link</label>
              <input
                id="merge-url-input"
                bind:value={forms.mergeUrl}
                maxlength="240"
                placeholder="Merge commit or release URL"
              />
            </div>
          </PullRequestReview>
        {:else if mode === 'merge-capability'}
          {#if step === 0}
            <h2>Which member?</h2>
            <select bind:value={forms.targetAgent}>
              <option value="" disabled>Select project member</option>
              {#each g.candidates as candidate (candidate)}
                <option value={candidate}>{name(candidate)}</option>
              {/each}
            </select>
          {:else if step === 1}
            <h2>Grant or revoke merge capability?</h2>
            <select bind:value={forms.action}>
              <option value="grant">Grant merge capability</option>
              <option value="revoke">Revoke merge capability</option>
            </select>
          {:else}
            <h2>Submit this change for a vote</h2>
            <div class="review-card">
              <strong>{forms.action === 'grant' ? 'Grant' : 'Revoke'} merge capability</strong>
              <span>{forms.targetAgent ? name(forms.targetAgent) : 'Selected member'}</span>
            </div>
          {/if}
        {:else if mode === 'repository-replacement'}
          {#if step === 0}
            <h2>Which blocked pull request triggered this?</h2>
            <select bind:value={forms.relatedPullRequest}>
              <option value="" disabled>Select blocked pull request</option>
              {#each replaceable as blocked (blocked.id)}
                <option value={blocked.id}>{blocked.pullRequestId} · {blocked.stageLabel}</option>
              {/each}
            </select>
          {:else if step === 1}
            <h2>What is the replacement repository URL?</h2>
            <input
              bind:value={forms.repositoryUrl}
              maxlength="240"
              placeholder="Replacement repository URL"
            />
          {:else if step === 2}
            <h2>Why does the repository need replacing?</h2>
            <textarea
              bind:value={forms.reason}
              rows="5"
              placeholder="Explain why the official repository must change"></textarea>
          {:else}
            <h2>Submit repository replacement for a vote</h2>
            <div class="review-card">
              <strong>{forms.repositoryUrl}</strong>
              <span>
                Blocked PR: {replaceable.find((p) => p.id === forms.relatedPullRequest)
                  ?.pullRequestId ?? forms.relatedPullRequest}
              </span>
              <p>{forms.reason}</p>
            </div>
          {/if}
        {/if}
        {#if message && (mode === 'create-pr' || mode === 'record-merge')}
          <p class="validation-copy" role="status">{message}</p>
        {/if}
        {#if sw.error}
          <p class="validation-copy" role="alert">{sw.error}</p>
        {/if}
      </div>
    </div>
  </PlanWizardShell>
{/if}
