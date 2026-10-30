<script lang="ts">
  import PeoplePicker from '#lib/components/create/PeoplePicker.svelte';
  import type { Contact } from '#lib/domain/messages.ts';
  import { inviteLink, parseInvite } from '#lib/domain/scopes.ts';
  import { contacts } from '#lib/services/messages.ts';
  import { myAgent } from '#lib/services/people.ts';

  import type { ScopeState } from './scopeState.svelte.ts';

  import '#lib/components/styles/create.css';

  /**
   * web's CommunityInvitePanel for a private community: members copy its invite link or code
   * and invite people by name; others paste a link or code to join. The link carries the
   * community's seed in its fragment, which never reaches a server (ADR 0068 decision 5).
   */
  interface Props {
    s: ScopeState;
    onRedeem: () => void;
    onInvite: (contact: Contact) => void;
  }

  let { s, onRedeem, onInvite }: Props = $props();

  const scope = $derived(s.scope);
  const isMember = $derived(!!s.project?.amMember);
  const link = $derived(
    scope && typeof window !== 'undefined'
      ? inviteLink(window.location.origin, scope.slug, scope.id)
      : '',
  );

  async function copy(value: string, done: string): Promise<void> {
    if (!value) return;
    if (!navigator.clipboard) {
      s.inviteFeedback = 'Copy the value manually from the field below.';
      s.inviteWarning = false;
      return;
    }
    await navigator.clipboard.writeText(value);
    s.inviteFeedback = done;
    s.inviteWarning = false;
  }

  let people: Promise<Contact[]> | undefined;
  /** People to invite: not yourself, not already in. */
  async function search(query: string): Promise<Contact[]> {
    people ??= Promise.all([contacts(), myAgent()]).then(([all, me]) =>
      all.filter((c) => c.agent !== me),
    );
    const inside = new Set(s.project?.members ?? []);
    const needle = query.trim().toLowerCase();
    const found = (await people)
      .filter((c) => !inside.has(c.agent) && c.name.toLowerCase().includes(needle))
      .slice(0, 6);
    // Someone who just signed up may not be listed yet: read the directory again next time.
    if (found.length === 0) people = undefined;
    return found;
  }
</script>

{#if scope}
  <section class="invite-card popover">
    {#if isMember}
      <p class="invite-kicker">Invite people</p>

      <div class="invite-section">
        <label class="field-label" for={`invite-link-${scope.slug}`}>Invite link</label>
        <div class="invite-actions">
          <input
            id={`invite-link-${scope.slug}`}
            aria-label={`${scope.handle} invite link`}
            readonly
            type="text"
            value={link}
          />
          <button
            class="action-chip"
            disabled={!link}
            type="button"
            onclick={() => void copy(link, 'Invite link copied.')}>Copy</button
          >
        </div>
      </div>

      <div class="invite-section">
        <label class="field-label" for={`invite-code-${scope.slug}`}>Invite code</label>
        <div class="invite-actions">
          <input
            id={`invite-code-${scope.slug}`}
            aria-label={`${scope.handle} invite code`}
            readonly
            type="text"
            value={scope.id}
          />
          <button
            class="action-chip"
            type="button"
            onclick={() => void copy(scope.id, 'Invite code copied.')}>Copy</button
          >
        </div>
      </div>

      <div class="invite-section">
        <PeoplePicker
          label="Invite by username"
          placeholder="Type a username"
          helperText="Search by username and send a direct invite."
          selected={[]}
          {search}
          onAdd={onInvite}
          onRemove={() => {}}
        />
      </div>
    {:else}
      <p class="invite-kicker">Join with invite</p>
      <p class="invite-copy">Paste an invite link or invite code from a community member.</p>

      <div class="invite-actions">
        <input
          aria-label={`${scope.handle} invite input`}
          bind:value={s.inviteDraft}
          placeholder="Paste invite link or invite code"
          type="text"
        />
        <button
          class="action-chip"
          disabled={!parseInvite(s.inviteDraft) || s.invitePending}
          type="button"
          onclick={onRedeem}>Join</button
        >
      </div>
    {/if}

    {#if s.inviteFeedback}
      <p class:warning={s.inviteWarning} class="invite-feedback">{s.inviteFeedback}</p>
    {/if}
  </section>
{/if}
