<script lang="ts">
  import SuggestionList from '#lib/components/messages/SuggestionList.svelte';
  import ConfirmDialog from '#lib/components/overlay/ConfirmDialog.svelte';
  import type { Contact } from '#lib/domain/messages.ts';

  import type { ScopeGovernance } from './scopeGovernance.svelte.ts';

  /**
   * web's "Who decides" section of a private community's settings (ADR 0068): organizers add
   * organizers and remove members at once; in a collaborative one, removal goes to a vote.
   * web's switches between the two (to_organizer / to_collaborative) aren't ported: the seed
   * fixes the governance (decision 4).
   */
  interface Props {
    organized: boolean;
    /** Organizer of an organizer-run community: changes apply straight away. */
    direct: boolean;
    isMember: boolean;
    organizerNames: string[];
    /** Members other than the viewer, by name. */
    members: Contact[];
    gov: ScopeGovernance;
    onChanged: (notice: string) => void;
  }

  let { organized, direct, isMember, organizerNames, members, gov, onChanged }: Props = $props();

  let organizerDraft = $state('');
  let removeDraft = $state('');
  let organizerPick = $state<Contact | null>(null);
  let removePick = $state<Contact | null>(null);
  let confirming = $state(false);

  /** Names repeat (app usernames aren't unique), so a pick names the person. */
  const matches = (q: string): Contact[] => {
    const needle = q.trim().toLowerCase();
    return needle ? members.filter((m) => m.name.toLowerCase().includes(needle)).slice(0, 6) : [];
  };
  const resolved = (draft: string, pick: Contact | null): Contact | null => {
    if (pick && pick.name === draft) return pick;
    const exact = members.filter((m) => m.name.toLowerCase() === draft.trim().toLowerCase());
    return exact.length === 1 ? (exact[0] ?? null) : null;
  };
  const organizerTarget = $derived(resolved(organizerDraft, organizerPick));
  const removeTarget = $derived(resolved(removeDraft, removePick));

  async function addOrganizer(): Promise<void> {
    if (!organizerTarget) return;
    if (await gov.addOrganizer(organizerTarget.agent)) onChanged('Organizer added');
    organizerDraft = '';
  }

  async function remove(): Promise<void> {
    if (!removeTarget) return;
    if (await gov.removeMember(removeTarget.agent)) {
      onChanged(direct ? 'Member removed' : 'Vote started');
    }
    removeDraft = '';
  }
</script>

<section class="governance">
  <h3>Who decides</h3>
  <p>
    {organized
      ? `Organizers: ${organizerNames.join(', ') || 'none'}`
      : 'Collaborative: all members vote on changes.'}
  </p>

  {#if direct}
    <div class="inline-field">
      <input
        bind:value={organizerDraft}
        oninput={() => (organizerPick = null)}
        placeholder="Username to make organizer"
      />
      <button type="button" disabled={gov.busy || !organizerTarget} onclick={addOrganizer}
        >Add</button
      >
    </div>
    {#if !organizerTarget}
      <SuggestionList
        items={matches(organizerDraft)}
        query={organizerDraft}
        emptyCopy="No matching members."
        onSelect={(c) => ((organizerPick = c), (organizerDraft = c.name))}
      />
    {/if}
  {/if}

  {#if direct || (!organized && isMember)}
    <div class="inline-field">
      <input
        bind:value={removeDraft}
        oninput={() => (removePick = null)}
        placeholder="Username to remove"
      />
      <button
        type="button"
        disabled={gov.busy || !removeTarget}
        onclick={() => (direct ? (confirming = true) : void remove())}
        >{direct ? 'Remove' : 'Propose'}</button
      >
    </div>
    {#if !removeTarget}
      <SuggestionList
        items={matches(removeDraft)}
        query={removeDraft}
        emptyCopy="No matching members."
        onSelect={(c) => ((removePick = c), (removeDraft = c.name))}
      />
    {/if}
  {/if}
</section>

<ConfirmDialog
  open={confirming}
  title={`Remove ${removeTarget?.name ?? 'this member'}?`}
  body="They stop being a member straight away, but anyone who still has the invite link can join again."
  confirmLabel="Remove"
  danger
  onSettle={(ok) => {
    confirming = false;
    if (ok) void remove();
  }}
/>
