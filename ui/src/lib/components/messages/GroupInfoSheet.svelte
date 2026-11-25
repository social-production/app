<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { resolve } from '$app/paths';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import { GROUP_TITLE_MAX, type Contact } from '#lib/domain/messages.ts';
  import GroupMemberRow from './GroupMemberRow.svelte';
  import '#lib/components/styles/ux/groupedit.css';

  /**
   * A group's page, as the Rail and Group Chat mockup §3 draws it (owner 2026-11-25): a top bar
   * with Cancel and Save, the group's face, its name, then its members with their role and a ⋮
   * menu for managers, and Leave group. Everyone else sees Group info without the controls.
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
    onLeave?: () => Promise<void>;
    onClose: () => void;
  }

  let p: Props = $props();

  const isManager = (agent: string) => agent === p.manager;
  const managing = $derived(isManager(p.me));
  const managerName = $derived(p.members.find((m) => isManager(m.agent))?.name ?? '');
  let draft = $state('');
  let adding = $state(false);
  let query = $state('');
  let found = $state<Contact[]>([]);
  let openMenu = $state<string | null>(null);
  let leaving = $state(false);
  let feedback = $state<{ text: string; ok: boolean } | null>(null);

  $effect(() => {
    if (!p.open) {
      draft = p.title;
      adding = false;
      openMenu = null;
      leaving = false;
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
        Number(isManager(b.agent)) - Number(isManager(a.agent)) ||
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

  async function leave(): Promise<void> {
    if (!leaving) {
      leaving = true;
      return;
    }
    await run(() => p.onLeave?.() ?? Promise.resolve(), 'You left the group.', 'Could not leave.');
    if (feedback?.ok) p.onClose();
  }

  function keys(e: KeyboardEvent): void {
    if (e.key !== 'Escape') return;
    if (openMenu) openMenu = null;
    else p.onClose();
  }
</script>

<svelte:window onkeydown={p.open ? keys : undefined} />

{#if p.open}
  <div
    class="group-page"
    role="dialog"
    aria-modal="true"
    aria-label={managing ? t('Edit group') : t('Group info')}
  >
    <header class="group-top">
      <button class="group-top-side" type="button" onclick={p.onClose}
        >{managing ? t('Cancel') : t('Close')}</button
      >
      <b>{managing ? t('Edit group') : t('Group info')}</b>
      {#if managing}
        <button
          class="group-top-side group-save"
          type="button"
          disabled={!renamed}
          onclick={() =>
            run(() => p.onRename(draft.trim()), 'Group name updated.', 'Could not rename it.')}
          >{t('Save')}</button
        >
      {:else}<span class="group-top-side"></span>{/if}
    </header>

    <div class="group-scroll">
      <div class="group-face"><AvatarBadge name={p.title} /></div>
      {#if managing}
        <label class="group-name-field">
          <span>{t('Group name')}</span>
          <input bind:value={draft} maxlength={GROUP_TITLE_MAX} type="text" />
        </label>
      {:else}
        <h2 class="group-title">{p.title}</h2>
        <p class="group-sub">
          {[
            `${p.members.length} members`,
            managerName ? t('managed by {name}', { name: managerName }) : '',
          ]
            .filter(Boolean)
            .join(' · ')}
        </p>
      {/if}

      <div class="group-section">
        <span>{t('Members')} · {p.members.length}</span>
      </div>
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
                  <button class="group-pill-button" type="button" onclick={() => add(contact)}
                    >{t('Add')}</button
                  >
                </div>
              {/each}
            </div>
          {/if}
        {/if}
        {#each ordered as member (member.agent)}
          <GroupMemberRow
            {member}
            you={member.agent === p.me}
            manager={isManager(member.agent)}
            canManage={managing && member.agent !== p.me}
            menuOpen={openMenu === member.agent}
            profile={resolve('/profile/[agent]', { agent: member.agent })}
            onMenu={(open) => (openMenu = open ? member.agent : null)}
            onRemove={() => {
              openMenu = null;
              void run(
                () => p.onRemove(member),
                `${member.name} was removed from the group.`,
                'Could not remove them.',
              );
            }}
          />
        {/each}
      </div>

      {#if feedback}
        <p class="group-feedback" class:ok={feedback.ok}>{feedback.text}</p>
      {/if}
      {#if p.onLeave}
        <button class="group-leave" type="button" onclick={() => void leave()}
          >{leaving ? t('Tap again to leave') : t('Leave group')}</button
        >
      {/if}
    </div>
  </div>
{/if}
