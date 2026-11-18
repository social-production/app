<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { resolve } from '$app/paths';
  import { bodyParts, formatByteSize } from '#lib/components/chat/chatFormat.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { standingLabel } from '#lib/components/people/standing.ts';
  import type { ChatAttachment, ChatLine } from '#lib/domain/chat.ts';
  import type { ChatSummary, ListPrefs } from '#lib/domain/messages.ts';
  import type { Person } from '#lib/domain/people.ts';
  import { getPerson } from '#lib/services/people.ts';

  /**
   * Telegram's chat page (Chats, Search and Reports mockups §2): tap a chat's face or name.
   * A direct chat shows the person (photo, membership, quick actions, bio and key); a group
   * its members. Then the photos, files and links shared in this chat, and for a person the
   * groups you share. Tapping a photo, file row or link row's date jumps to its message.
   */
  interface Props {
    chat: ChatSummary;
    title: string;
    me: string;
    names: Record<string, string>;
    lines: ChatLine[];
    /** Your groups that this person is also in (direct chats). */
    groups: { id: string; title: string; count: number }[];
    fileUrl: (attachment: ChatAttachment) => Promise<string>;
    onBack: () => void;
    onMenu: (x: number, y: number) => void;
    onCall: () => void;
    onPrefs: (prefs: ListPrefs) => void;
    onEditGroup: () => void;
    onJump: (id: string) => void;
    onOpenChat: (id: string) => void;
  }

  let p: Props = $props();

  const group = $derived(p.chat.kind === 'group');
  const partner = $derived(group ? null : (p.chat.members.find((m) => m !== p.me) ?? null));
  let person = $state<Person | null>(null);
  $effect(() => {
    const agent = partner;
    if (agent)
      void getPerson(agent).then(
        (found) => (person = found),
        () => undefined,
      );
  });

  const tabs = $derived(
    group ? ['Members', 'Media', 'Files', 'Links'] : ['Media', 'Files', 'Links', 'Groups'],
  );
  let picked = $state<string | null>(null);
  const tab = $derived(picked ?? tabs[0]);

  const shared = $derived(p.lines.filter((l) => l.kind === 'text' && !l.pending && !l.moderation));
  const photos = $derived(
    shared
      .flatMap((l) => l.attachments.filter((a) => a.kind === 'image').map((a) => ({ line: l, a })))
      .reverse(),
  );
  const files = $derived(
    shared
      .flatMap((l) => l.attachments.filter((a) => a.kind !== 'image').map((a) => ({ line: l, a })))
      .reverse(),
  );
  const links = $derived(
    shared
      .flatMap((l) => bodyParts(l.body).flatMap((part) => (part.href ? [{ line: l, part }] : [])))
      .reverse(),
  );
  /** A link's site ("seedsavers.org.au"), or its text for a link inside the app. */
  function site(href: string, text: string): string {
    try {
      return href.startsWith('/') ? text : new URL(href).hostname.replace(/^www\./, '');
    } catch {
      return text;
    }
  }
  const day = (at: number): string =>
    new Date(at).toLocaleDateString(undefined, { day: 'numeric', month: 'short' });

  function photoSrc(node: HTMLImageElement, a: ChatAttachment): void {
    void p.fileUrl(a).then(
      (url) => (node.src = url),
      () => undefined,
    );
  }

  async function download(a: ChatAttachment): Promise<void> {
    const url = await p.fileUrl(a).catch(() => null);
    if (!url) return;
    const link = document.createElement('a');
    link.href = url;
    link.download = a.name;
    link.click();
  }
</script>

{#snippet tile(icon: string, label: string, run: () => void)}
  <button class="about-tile" type="button" onclick={run}
    ><ToolbarIcon name={icon} />{t(label)}</button
  >
{/snippet}

<section class="chat-about" aria-label={`About ${p.title}`}>
  <div class="about-top">
    <button class="about-round" type="button" aria-label={t('Back to the chat')} onclick={p.onBack}>
      <svg aria-hidden="true" viewBox="0 0 24 24" width="22" height="22"
        ><path
          d="M15 5l-7 7 7 7"
          fill="none"
          stroke="currentColor"
          stroke-width="2.4"
          stroke-linecap="round"
          stroke-linejoin="round"
        /></svg
      >
    </button>
    <button
      class="about-round"
      type="button"
      aria-label={t('Chat menu')}
      onclick={(event) => {
        const box = event.currentTarget.getBoundingClientRect();
        p.onMenu(box.right - 220, box.bottom + 6);
      }}><ToolbarIcon name="kebab" /></button
    >
  </div>

  <div class="about-scroll">
    <div class="about-head">
      <span class="about-face"
        ><AvatarBadge name={p.title} avatar={person?.profile?.avatar ?? null} /></span
      >
      <h2>{p.title}</h2>
      <p class="about-sub">
        {#if group}{p.chat.members.length} members{#if p.chat.manager === p.me}
            {t('· you manage it')}{/if}
        {:else if person}{t(standingLabel(person))}{#if person.profile?.home_area}
            · {person.profile.home_area}{/if}{/if}
      </p>
      <div class="about-tiles">
        {#if group}
          {@render tile('pencil', p.chat.manager === p.me ? 'Edit' : 'Info', p.onEditGroup)}
        {:else}
          {@render tile('phone', 'Call', p.onCall)}
        {/if}
        {@render tile(p.chat.muted ? 'bell' : 'mute', p.chat.muted ? 'Unmute' : 'Mute', () =>
          p.onPrefs({ muted: !p.chat.muted }),
        )}
        {@render tile('pin', p.chat.pinned ? 'Unpin' : 'Pin', () =>
          p.onPrefs({ pinned: !p.chat.pinned }),
        )}
        {#if partner}
          <a class="about-tile" href={resolve('/profile/[agent]', { agent: partner })}
            ><ToolbarIcon name="user" />{t('Profile')}</a
          >
        {/if}
      </div>
      {#if partner && person?.profile?.bio}
        <!-- Detail Pages Cleanup §5: no key row; the full key stays in Settings › Account. -->
        <div class="about-info">
          <div>
            <span>{t('Bio')}</span>
            <p>{person.profile.bio}</p>
          </div>
        </div>
      {/if}
    </div>

    <div class="about-tabs" role="tablist" aria-label={t('Shared in this chat')}>
      {#each tabs as name (name)}
        <button
          type="button"
          role="tab"
          aria-selected={tab === name}
          class:on={tab === name}
          onclick={() => (picked = name)}>{name}</button
        >
      {/each}
    </div>

    {#if tab === 'Members'}
      {#each p.chat.members as agent (agent)}
        <a class="about-row" href={resolve('/profile/[agent]', { agent })}>
          <AvatarBadge size="md" name={p.names[agent] ?? '…'} />
          <span class="about-row-copy"
            ><b>{agent === p.me ? t('You') : (p.names[agent] ?? '…')}</b></span
          >
          {#if agent === p.chat.manager}<span class="about-chip">{t('Manager')}</span>{/if}
        </a>
      {/each}
    {:else if tab === 'Media'}
      {#if photos.length === 0}<p class="about-empty">{t('No photos in this chat yet.')}</p>{/if}
      <div class="about-grid">
        {#each photos as { line, a }, i (i)}
          <button
            type="button"
            aria-label={`Show ${a.name} in the chat`}
            onclick={() => p.onJump(line.id)}
          >
            <img alt={a.name} use:photoSrc={a} />
          </button>
        {/each}
      </div>
    {:else if tab === 'Files'}
      {#if files.length === 0}<p class="about-empty">{t('No files in this chat yet.')}</p>{/if}
      {#each files as { line, a }, i (i)}
        <button class="about-row" type="button" onclick={() => void download(a)}>
          <span class="about-glyph"><ToolbarIcon name="copy" /></span>
          <span class="about-row-copy"><b>{a.name}</b><span>{formatByteSize(a.size)}</span></span>
          <span class="about-date">{day(line.createdAt)}</span>
        </button>
      {/each}
    {:else if tab === 'Links'}
      {#if links.length === 0}<p class="about-empty">{t('No links in this chat yet.')}</p>{/if}
      {#each links as { line, part }, i (i)}
        <div class="about-row">
          <span class="about-glyph"><ToolbarIcon name="link" /></span>
          <!-- eslint-disable svelte/no-navigation-without-resolve -- a link written in the chat -->
          <a
            class="about-row-copy"
            href={part.href}
            target={part.internal ? undefined : '_blank'}
            rel="noopener noreferrer"
            ><b>{site(part.href ?? '', part.text)}</b><span>{part.href}</span></a
          >
          <!-- eslint-enable svelte/no-navigation-without-resolve -->
          <button class="about-date" type="button" onclick={() => p.onJump(line.id)}
            >{day(line.createdAt)}</button
          >
        </div>
      {/each}
    {:else}
      {#if p.groups.length === 0}<p class="about-empty">No groups with {p.title} yet.</p>{/if}
      {#each p.groups as g (g.id)}
        <button class="about-row" type="button" onclick={() => p.onOpenChat(g.id)}>
          <AvatarBadge size="md" name={g.title} />
          <span class="about-row-copy"><b>{g.title}</b><span>{g.count} members</span></span>
        </button>
      {/each}
    {/if}
  </div>
</section>
