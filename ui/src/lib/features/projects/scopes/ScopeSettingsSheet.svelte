<script lang="ts">
  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { APPROVAL_PERCENT } from '#lib/domain/governance.ts';
  import { relativeCompact } from '#lib/domain/time.ts';
  import type { ScopeKind } from '#lib/domain/scopes.ts';

  import { describeDecision, type ScopeGovernance } from './scopeGovernance.svelte.ts';

  import '#lib/components/styles/scopes.css';

  /**
   * web's ScopeSettingsSheet (the gear): Settings (name, description) and Decisions (open and
   * past votes). Public scopes are collaborative, so a change goes to a member vote. The name
   * is the handle tags use, so it stays (ADR 0068); scopes have no picture yet.
   */
  interface Props {
    open: boolean;
    kind: ScopeKind;
    handle: string;
    description: string;
    isMember: boolean;
    gov: ScopeGovernance;
    memberName: (agent: string) => string;
    onClose: () => void;
  }

  let { open, kind, handle, description, isMember, gov, memberName, onClose }: Props = $props();

  let tab = $state<'settings' | 'decisions'>('settings');
  let draft = $state('');
  let notice = $state('');

  $effect(() => {
    if (!open) return;
    draft = description;
    notice = '';
    void gov.load();
  });

  const changed = $derived(draft.trim() !== description.trim());

  const dateOf = (micros: number | null): string =>
    micros
      ? new Date(micros / 1000).toLocaleDateString(undefined, {
          day: 'numeric',
          month: 'short',
          year: 'numeric',
        })
      : '';

  async function save(): Promise<void> {
    if (await gov.proposeDescription(draft.trim())) {
      notice = 'Vote started';
      tab = 'decisions';
    }
  }
</script>

<OverlaySheet
  {open}
  title="Settings"
  labelledById="scope-settings-title"
  rootClass="scope-settings"
  {onClose}
>
  {#snippet toolbar()}
    <div class="sheet-tabs" role="tablist" aria-label="Settings sections">
      <button
        type="button"
        role="tab"
        aria-selected={tab === 'settings'}
        class:active={tab === 'settings'}
        onclick={() => (tab = 'settings')}>Settings</button
      >
      <button
        type="button"
        role="tab"
        aria-selected={tab === 'decisions'}
        class:active={tab === 'decisions'}
        onclick={() => (tab = 'decisions')}
      >
        Decisions
        {#if gov.open.length}<span class="tab-count">{gov.open.length}</span>{/if}
      </button>
    </div>
  {/snippet}
  <div class="settings-body">
    {#if gov.message}<p class="error">{gov.message}</p>{/if}
    {#if notice}<p class="how">{notice}</p>{/if}
    {#if tab === 'decisions'}
      <section class="decisions" aria-label="Decisions">
        <div class="group-head">
          <h3>Open decisions</h3>
          <span>{gov.open.length}</span>
        </div>
        {#if !gov.loaded}
          <p class="empty-card">Loading…</p>
        {:else if gov.open.length === 0}
          <p class="empty-card">No open decisions.</p>
        {:else}
          <div class="decision-list">
            {#each gov.open as item (item.id)}
              <article class="decision-card open">
                <div class="decision-row">
                  <strong>{describeDecision(item, memberName)}</strong>
                  <span>{item.required} yes needed · {APPROVAL_PERCENT}%</span>
                </div>
                <div class="decision-row">
                  <span>
                    {[`${item.yes} yes · ${item.no} no`, item.myVote && `you voted ${item.myVote}`]
                      .filter(Boolean)
                      .join(' · ')}
                  </span>
                  <span>{item.proposer} · {relativeCompact(item.createdAt)}</span>
                </div>
                {#if isMember && item.myVote === null}
                  <div class="vote-row">
                    <button
                      type="button"
                      disabled={gov.busy}
                      onclick={() => void gov.vote(item.id, 'Yes')}>Yes</button
                    >
                    <button
                      type="button"
                      disabled={gov.busy}
                      onclick={() => void gov.vote(item.id, 'No')}>No</button
                    >
                  </div>
                {:else if !isMember}
                  <p class="vote-note">Members vote on this. Join to take part.</p>
                {/if}
              </article>
            {/each}
          </div>
        {/if}

        <div class="group-head">
          <h3>Past decisions</h3>
          <span>{gov.past.length}</span>
        </div>
        {#if gov.past.length === 0}
          <p class="empty-card">No past decisions yet.</p>
        {:else}
          <div class="decision-list">
            {#each gov.past as item (item.id)}
              <article
                class="decision-card"
                class:approved={item.status === 'Passed'}
                class:rejected={item.status !== 'Passed'}
              >
                <div class="decision-row">
                  <strong>{describeDecision(item, memberName)}</strong>
                  <span>{item.status === 'Passed' ? 'Passed' : 'Rejected'}</span>
                </div>
                <div class="decision-row">
                  <span>{item.yes} yes · {item.no} no</span>
                  <span>{[item.proposer, dateOf(item.closedAt)].filter(Boolean).join(' · ')}</span>
                </div>
              </article>
            {/each}
          </div>
        {/if}
      </section>
    {:else}
      <section class="picture-row">
        <span class="scope-avatar" style="--avatar-size: 64px" aria-hidden="true">
          <SurfaceIcon icon={kind} size="md" />
        </span>
        <div class="picture-copy">
          <span>Shown on cards and at the top of the page.</span>
        </div>
      </section>

      <label class="field">
        <span>Name</span>
        <input value={handle} maxlength="32" aria-describedby="scope-name-hint" disabled />
        <span id="scope-name-hint" class="hint">Tags use this name, so it stays as it is.</span>
      </label>
      <label class="field">
        <span>Description</span>
        <textarea bind:value={draft} rows="4" maxlength="2000" disabled={!isMember}></textarea>
      </label>

      <p class="how">
        Collaborative: changes go to a member vote ({APPROVAL_PERCENT}% yes, with quorum based on
        members).
      </p>

      {#if isMember}
        <button class="primary" type="button" disabled={gov.busy || !changed} onclick={save}>
          Propose change
        </button>
      {/if}
    {/if}
  </div>
</OverlaySheet>
