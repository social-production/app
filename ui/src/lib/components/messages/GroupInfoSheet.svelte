<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { resolve } from '$app/paths';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { GROUP_TITLE_MAX, type Contact } from '#lib/domain/messages.ts';
  import '#lib/components/styles/ux/groupedit.css';

  /**
   * A group's page (Rail and Group Chat mockup §3, owner 2026-11-25): its face and name, then
   * its members with their role. The manager renames it, adds people and, from a member's ⋮,
   * removes them; everyone else sees the same page without the controls. Group photos and
   * more than one manager wait for the chat network's next reset.
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

  const managing = $derived(!!p.manager && p.manager === p.me);
  const managerName = $derived(p.members.find((m) => m.agent === p.manager)?.name ?? '');
  let draft = $state('');
  let adding = $state(false);
  let query = $state('');
  let found = $state<Contact[]>([]);
  let openMenu = $state<string | null>(null);
  let feedback = $state<{ text: string; ok: boolean } | null>(null);

  $effect(() => {
    if (!p.open) {
      draft = p.title;
      adding = false;
      openMenu = null;
      feedback = null;
    }
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
  const renamed = $derived(draft.trim() !== '' && draft.trim() !== p.title);
  const ordered = $derived(
    [...p.members].sort(
      (a, b) =>
        Number(b.agent === p.manager) - Number(a.agent === p.manager) ||
        Number(b.agent === p.me) - Number(a.agent === p.me),
    ),
  );

  async function run(action: () => Promise<void>, done: string, failed: string): Promise<void> {
    try {
      await action();
      feedback = { text: done, ok: true };
    } catch (e) {
      feedback = { text: e instanceof Error && e.message ? e.message : failed, ok: false };
    }
  }

  async function add(contact: Contact): Promise<void> {
    await run(() => p.onAdd(contact), `${contact.name} joined the group.`, 'Could not add them.');
    if (feedback?.ok) query = '';
  }

  async function remove(contact: Contact): Promise<void> {
    openMenu = null;
    await run(
      () => p.onRemove(contact),
      `${contact.name} was removed from the group.`,
      'Could not remove them.',
    );
  }
</script>

<OverlaySheet
  open={p.open}
  title={managing ? t('Edit group') : t('Group info')}
  rootClass="messages-page group-edit"
  onClose={p.onClose}
>
  <div class="group-edit-body">
    <div class="group-face"><AvatarBadge name={p.title} /></div>
    {#if managing}
      <label class="group-name-field">
        <span>{t('Group name')}</span>
        <span class="group-name-row">
          <input
            bind:value={draft}
            maxlength={GROUP_TITLE_MAX}
            type="text"
            aria-label={t('Group name')}
          />
          {#if renamed}
            <button
              type="button"
              class="group-save"
              onclick={() =>
                run(() => p.onRename(draft.trim()), 'Group name updated.', 'Could not rename it.')}
              >{t('Save')}</button
            >
          {/if}
        </span>
      </label>
    {:else}
      <h3 class="group-title">{p.title}</h3>
      <p class="group-sub">
        {[
          `${p.members.length} members`,
          managerName ? t('managed by {name}', { name: managerName }) : '',
        ]
          .filter(Boolean)
          .join(' · ')}
      </p>
    {/if}

    <div class="group-section"><span>{t('Members')} · {p.members.length}</span></div>
    <div class="group-list">
      {#if managing}
        <button class="group-row group-add" type="button" onclick={() => (adding = !adding)}>
          <span class="group-plus" aria-hidden="true">+</span><span>{t('Add people')}</span>
        </button>
        {#if adding}
          <div class="group-search">
            <input bind:value={query} placeholder={t('Type a username')} type="text" />
            {#each addable as contact (contact.agent)}
              <div class="group-row">
                <AvatarBadge size="sm" name={contact.name} agent={contact.agent} />
                <b>{contact.name}</b>
                <button class="group-chip-button" type="button" onclick={() => add(contact)}
                  >{t('Add')}</button
                >
              </div>
            {/each}
          </div>
        {/if}
      {/if}
      {#each ordered as member (member.agent)}
        <div class="group-row">
          <a class="group-person" href={resolve('/profile/[agent]', { agent: member.agent })}>
            <AvatarBadge size="md" name={member.name} agent={member.agent} />
            <b>{member.name}{member.agent === p.me ? t(' (you)') : ''}</b>
          </a>
          {#if member.agent === p.manager}
            <span class="group-role">{t('Manager')}</span>
          {:else if managing && member.agent !== p.me}
            <button
              class="group-more"
              type="button"
              aria-label={t('More for {name}', { name: member.name })}
              aria-expanded={openMenu === member.agent}
              onclick={() => (openMenu = openMenu === member.agent ? null : member.agent)}>⋮</button
            >
          {/if}
        </div>
        {#if openMenu === member.agent}
          <div class="group-menu" role="menu">
            <a role="menuitem" href={resolve('/profile/[agent]', { agent: member.agent })}
              >{t('View profile')}</a
            >
            <button role="menuitem" class="danger" type="button" onclick={() => remove(member)}
              >{t('Remove from group')}</button
            >
          </div>
        {/if}
      {/each}
    </div>

    {#if feedback}
      <p class:success={feedback.ok} class="composer-feedback">{feedback.text}</p>
    {/if}
  </div>
</OverlaySheet>
