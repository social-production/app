<script lang="ts">
  import { resolve } from '$app/paths';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { GROUP_TITLE_MAX, type Contact } from '#lib/domain/messages.ts';

  /**
   * web's "Chat info" sheet for a group: rename it, see who is in it (and who manages it), add
   * and remove people. Only the manager's changes go through; others get web's message back.
   */
  interface Props {
    open: boolean;
    title: string;
    me: string;
    manager: string | null;
    members: Contact[];
    search: (query: string) => Promise<Contact[]>;
    onRename: (title: string) => Promise<void>;
    onAdd: (contact: Contact) => Promise<void>;
    onRemove: (contact: Contact) => Promise<void>;
    onClose: () => void;
  }

  let p: Props = $props();

  let draft = $state('');
  let query = $state('');
  let found = $state<Contact[]>([]);
  let mode = $state<'add' | 'remove' | null>(null);
  let feedback = $state<{ text: string; ok: boolean } | null>(null);

  // web keeps the field in step with the group's name while the sheet is shut.
  $effect(() => {
    if (!p.open) draft = p.title;
  });

  $effect(() => {
    const q = query.trim();
    if (!q) {
      found = [];
      return;
    }
    let live = true;
    void p.search(q).then(
      (list) => live && (found = list),
      () => live && (found = []),
    );
    return () => {
      live = false;
    };
  });

  const addable = $derived(found.filter((c) => !p.members.some((m) => m.agent === c.agent)));
  const removable = $derived(p.members.filter((m) => m.agent !== p.me));

  async function run(action: () => Promise<void>, done: string, failed: string): Promise<void> {
    try {
      await action();
      feedback = { text: done, ok: true };
    } catch (e) {
      feedback = { text: e instanceof Error && e.message ? e.message : failed, ok: false };
    }
  }

  function toggle(next: 'add' | 'remove'): void {
    mode = mode === next ? null : next;
    feedback = null;
  }

  async function add(contact: Contact): Promise<void> {
    await run(
      () => p.onAdd(contact),
      `${contact.name} joined the group chat.`,
      'That member could not be added.',
    );
    if (feedback?.ok) query = '';
  }
</script>

<!-- The sheet leaves the page (portal): rootClass keeps web's MessagesPage styles on it. -->
<OverlaySheet open={p.open} title="Chat info" rootClass="messages-page" onClose={p.onClose}>
  <section class="group-settings-card">
    <label class="composer-field">
      <span>Group name</span>
      <div class="inline-field">
        <input
          bind:value={draft}
          maxlength={GROUP_TITLE_MAX}
          placeholder="Rename group chat"
          type="text"
        />
        <button
          class="secondary-button"
          type="button"
          onclick={() =>
            run(
              () => p.onRename(draft),
              'Group name updated.',
              'The group name could not be updated.',
            )}>Save</button
        >
      </div>
    </label>

    <div class="composer-field">
      <span>Members</span>
      <div class="member-links">
        {#each p.members as member (member.agent)}
          <a class="member-link" href={resolve('/profile/[agent]', { agent: member.agent })}
            >{member.name}{member.agent === p.me ? ' (you)' : ''}{member.agent === p.manager
              ? ' · manages the group'
              : ''}</a
          >
        {/each}
      </div>
    </div>

    <div class="contact-list">
      <button
        class:active={mode === 'add'}
        class="contact-chip"
        type="button"
        onclick={() => toggle('add')}>Add member</button
      >
      <button
        class:active={mode === 'remove'}
        class="contact-chip"
        type="button"
        onclick={() => toggle('remove')}>Remove member</button
      >
    </div>

    {#if mode === 'add'}
      <div class="composer-field">
        <span>Add someone</span>
        <input
          bind:value={query}
          placeholder="Type a username"
          type="text"
          onkeydown={(event) => {
            const first = addable[0];
            if (event.key === 'Enter' && first) {
              event.preventDefault();
              void add(first);
            }
          }}
        />
        {#if addable.length > 0}
          <div class="contact-list">
            {#each addable as contact (contact.agent)}
              <button class="contact-chip" type="button" onclick={() => add(contact)}
                >{contact.name}</button
              >
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    {#if mode === 'remove'}
      <div class="composer-field">
        <span>Remove someone</span>
        <div class="contact-list">
          {#each removable as member (member.agent)}
            <button
              class="contact-chip"
              type="button"
              onclick={() =>
                run(
                  () => p.onRemove(member),
                  `${member.name} was removed from the group chat.`,
                  'That member could not be removed.',
                )}>{member.name}</button
            >
          {/each}
        </div>
      </div>
    {/if}

    {#if feedback}
      <p class:success={feedback.ok} class="composer-feedback">{feedback.text}</p>
    {/if}
  </section>
</OverlaySheet>
