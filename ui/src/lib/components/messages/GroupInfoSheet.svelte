<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { resolve } from '$app/paths';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import { GROUP_TITLE_MAX, type Contact } from '#lib/domain/messages.ts';
  import GroupMemberRow from './GroupMemberRow.svelte';
  import '#lib/components/styles/ux/groupedit.css';

  /**
   * A group's page, as the Rail and Group Chat mockup §3 draws it (owner 2026-11-25): a top bar
   * with Cancel and Save, the group's photo (Change photo for managers), its name, then its
   * members with their role and a ⋮ menu for managers (make or stop a manager, remove; ADR
   * 0122), and Leave group. Everyone else sees Group info without the controls.
   */
  interface Props {
    open: boolean;
    title: string;
    me: string;
    manager: string | null;
    managers: string[];
    picture: string | null;
    members: Contact[];
    search: (query: string) => Promise<Contact[]>;
    onRename: (title: string) => Promise<void>;
    onAdd: (contact: Contact) => Promise<void>;
    onRemove: (contact: Contact) => Promise<void>;
    onManager: (contact: Contact, on: boolean) => Promise<void>;
    onPicture: (photo: Blob) => Promise<void>;
    /** The shared crop screen: the placed photo, or `null` when closed. */
    crop: (file: File) => Promise<Blob | null>;
    onLeave?: () => Promise<void>;
    onClose: () => void;
  }

  let p: Props = $props();

  const isManager = (agent: string) =>
    p.managers.length > 0 ? p.managers.includes(agent) : agent === p.manager;
  const managing = $derived(isManager(p.me));
  const managerNames = $derived(
    p.members
      .filter((m) => isManager(m.agent))
      .map((m) => m.name)
      .join(` ${t('and')} `),
  );
  const managerCount = $derived(p.members.filter((m) => isManager(m.agent)).length);
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
      preview = null;
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

  let picking = $state<HTMLInputElement | null>(null);
  let preview = $state<string | null>(null);
  const face = $derived(preview ?? p.picture);
  async function choose(file: File | undefined): Promise<void> {
    if (!file) return;
    const placed = await p.crop(file);
    if (!placed) return;
    // The placed photo shows at once while it saves.
    const url = URL.createObjectURL(placed);
    preview = url;
    feedback = { text: t('Saving photo…'), ok: true };
    await run(() => p.onPicture(placed), 'Group photo updated.', 'Could not save it.');
    if (!feedback?.ok) preview = null;
  }

  async function grant(member: Contact, on: boolean): Promise<void> {
    openMenu = null;
    await run(
      () => p.onManager(member, on),
      on ? `${member.name} manages the group too.` : `${member.name} no longer manages it.`,
      'Could not change that.',
    );
  }

  // Save keeps a new name, then closes; a photo or manager change is already saved.
  let saving = $state(false);
  async function save(): Promise<void> {
    if (!renamed) return p.onClose();
    saving = true;
    await run(() => p.onRename(draft.trim()), 'Group name updated.', 'Could not rename it.');
    saving = false;
    if (feedback?.ok) p.onClose();
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
          disabled={saving}
          onclick={() => void save()}>{t('Save')}</button
        >
      {:else}<span class="group-top-side"></span>{/if}
    </header>

    <div class="group-scroll">
      <div class="group-face">
        <span class="group-face-inner">
          {#if face}<img src={face} alt="" />{:else}<AvatarBadge name={p.title} />{/if}
          {#if managing}
            <button
              class="group-camera"
              type="button"
              aria-label={t('Change photo')}
              onclick={() => picking?.click()}
            >
              <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true"
                ><path
                  d="M4 8h3l2-3h6l2 3h3v11H4z"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.2"
                  stroke-linejoin="round"
                /><circle
                  cx="12"
                  cy="13"
                  r="3.5"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.2"
                /></svg
              >
            </button>
          {/if}
        </span>
        {#if managing}
          <button class="group-change" type="button" onclick={() => picking?.click()}
            >{t('Change photo')}</button
          >
          <input
            bind:this={picking}
            hidden
            type="file"
            accept="image/*"
            onchange={(e) => {
              void choose(e.currentTarget.files?.[0]);
              e.currentTarget.value = '';
            }}
          />
        {/if}
      </div>
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
            managerNames ? t('managed by {name}', { name: managerNames }) : '',
          ]
            .filter(Boolean)
            .join(' · ')}
        </p>
      {/if}

      <div class="group-section">
        <span>{t('Members')} · {p.members.length}</span>
        {#if managerCount > 1}<span>{managerCount} {t('managers')}</span>{/if}
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
            keepSlot={managing}
            menuOpen={openMenu === member.agent}
            profile={resolve('/profile/[agent]', { agent: member.agent })}
            onMenu={(open) => (openMenu = open ? member.agent : null)}
            onManager={() => void grant(member, !isManager(member.agent))}
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
