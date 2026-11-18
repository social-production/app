<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import PlanWizardShell from '#lib/components/plans/PlanWizardShell.svelte';

  import PullRequestSheet from './PullRequestSheet.svelte';
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

{#if (mode === 'vote-pr' || mode === 'record-merge') && pr}
  <PullRequestSheet
    {pr}
    {mode}
    names={sw.names}
    busy={sw.busy}
    canSubmit={canGoNext}
    onVote={(c) => void vote(c)}
    onSubmit={() => void next()}
    onClose={() => sw.close()}
  >
    <div class="vote-fields">
      <label class="vote-field-name" for="merge-id-input">{t('Merge ID')}</label>
      <input
        id="merge-id-input"
        bind:value={forms.mergeId}
        maxlength="120"
        placeholder={t('Merge commit or release ID')}
      />
      <label class="vote-field-name" for="merge-url-input">{t('Merge link')}</label>
      <input
        id="merge-url-input"
        bind:value={forms.mergeUrl}
        maxlength="240"
        placeholder={t('Merge commit or release URL')}
      />
    </div>
    {#if sw.error}<p class="validation-copy" role="alert">{sw.error}</p>{/if}
  </PullRequestSheet>
{:else if mode && g}
  <PlanWizardShell
    open={true}
    title={wizardTitle(mode, pr)}
    stepIndex={step}
    stepCount={count}
    nextLabel={step >= count - 1 ? submitLabel(mode) : t('Next')}
    canGoBack={step > 0}
    {canGoNext}
    onClose={() => sw.close()}
    onDismiss={() => sw.close()}
    onBack={() => (step = Math.max(step - 1, 0))}
    onNext={() => void next()}
  >
    <div class="software-wizard">
      <div class="question-block">
        {#if mode === 'create-pr'}
          {#if step === 0}
            <h2>{t('What should we call this pull request?')}</h2>
            <p class="helper-copy">
              {t('Use a short title that matches the change people will vote on.')}
            </p>
            <input bind:value={forms.title} maxlength="120" placeholder={t('Pull request title')} />
          {:else if step === 1}
            <h2>{t('Where is the pull request?')}</h2>
            <p class="helper-copy">{t('Link the external PR so voters can inspect the change.')}</p>
            <div class="field-grid">
              <input
                bind:value={forms.pullRequestId}
                maxlength="40"
                placeholder={t('PR number or ID')}
              />
              <input
                bind:value={forms.pullRequestUrl}
                maxlength="240"
                placeholder={t('Pull request URL')}
              />
            </div>
          {:else if step === 2}
            <h2>{t('What does this change?')}</h2>
            <p class="helper-copy">
              {t('Summarise the effect in plain language for non-specialist voters.')}
            </p>
            <textarea
              bind:value={forms.summary}
              rows="5"
              placeholder={t('What does this pull request change?')}></textarea>
          {:else}
            <h2>{t('Ready to submit?')}</h2>
            <div class="review-card">
              <strong>{forms.title}</strong>
              <span>{forms.pullRequestId} · {forms.pullRequestUrl}</span>
              <p>{forms.summary}</p>
            </div>
          {/if}
        {:else if mode === 'merge-capability'}
          {#if step === 0}
            <h2>{t('Which member?')}</h2>
            <select bind:value={forms.targetAgent}>
              <option value="" disabled>{t('Select project member')}</option>
              {#each g.candidates as candidate (candidate)}
                <option value={candidate}>{name(candidate)}</option>
              {/each}
            </select>
          {:else if step === 1}
            <h2>{t('Grant or revoke merge capability?')}</h2>
            <select bind:value={forms.action}>
              <option value="grant">{t('Grant merge capability')}</option>
              <option value="revoke">{t('Revoke merge capability')}</option>
            </select>
          {:else}
            <h2>{t('Submit this change for a vote')}</h2>
            <div class="review-card">
              <strong>{forms.action === 'grant' ? t('Grant') : t('Revoke')} merge capability</strong
              >
              <span>{forms.targetAgent ? name(forms.targetAgent) : t('Selected member')}</span>
            </div>
          {/if}
        {:else if mode === 'repository-replacement'}
          {#if step === 0}
            <h2>{t('Which blocked pull request triggered this?')}</h2>
            <select bind:value={forms.relatedPullRequest}>
              <option value="" disabled>{t('Select blocked pull request')}</option>
              {#each replaceable as blocked (blocked.id)}
                <option value={blocked.id}>{blocked.pullRequestId} · {t(blocked.stageLabel)}</option
                >
              {/each}
            </select>
          {:else if step === 1}
            <h2>{t('What is the replacement repository URL?')}</h2>
            <input
              bind:value={forms.repositoryUrl}
              maxlength="240"
              placeholder={t('Replacement repository URL')}
            />
          {:else if step === 2}
            <h2>{t('Why does the repository need replacing?')}</h2>
            <textarea
              bind:value={forms.reason}
              rows="5"
              placeholder={t('Explain why the official repository must change')}></textarea>
          {:else}
            <h2>{t('Submit repository replacement for a vote')}</h2>
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
        {#if message && mode === 'create-pr'}
          <p class="validation-copy" role="status">{message}</p>
        {/if}
        {#if sw.error}
          <p class="validation-copy" role="alert">{sw.error}</p>
        {/if}
      </div>
    </div>
  </PlanWizardShell>
{/if}
