<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import ChatMarks from '#lib/components/messages/ChatMarks.svelte';
  import PinBar from '#lib/components/messages/PinBar.svelte';
  import type { ChatPin, ChatSummary } from '#lib/domain/messages.ts';
  import type { Avatar } from '#lib/domain/people.ts';
  import { getPerson } from '#lib/services/people.ts';

  /**
   * An open chat's header as Telegram draws it (owner, 2026-11-16): back; the face and name,
   * which open the chat's page; call (direct chats) and ⋮ in one capsule; the pinned messages
   * below.
   */
  interface Props {
    chat: ChatSummary;
    title: string;
    me: string;
    pins: ChatPin[];
    onBack: () => void;
    onAbout: () => void;
    onCall: () => void;
    onMenu: (x: number, y: number) => void;
    onJump: (id: string) => void;
    onUnpin: (id: string) => void;
  }

  let p: Props = $props();

  // A direct chat shows the person's photo, as Telegram does.
  let avatar = $state<Avatar | null>(null);
  $effect(() => {
    const partner = p.chat.kind === 'direct' ? p.chat.members.find((m) => m !== p.me) : null;
    if (partner)
      void getPerson(partner).then(
        (person) => (avatar = person.profile?.avatar ?? null),
        () => undefined,
      );
  });
</script>

<header class="chat-header">
  <button
    class="back-button"
    type="button"
    aria-label={t('Back to chats')}
    title={t('Back to chats')}
    onclick={p.onBack}
  >
    <svg aria-hidden="true" viewBox="0 0 24 24" width="22" height="22">
      <path
        d="M15 5l-7 7 7 7"
        fill="none"
        stroke="currentColor"
        stroke-width="2.4"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>
  <div class="chat-identity">
    <button
      class="identity-trigger"
      type="button"
      aria-label={`About ${p.title}`}
      onclick={p.onAbout}
    >
      <div>
        <h2>
          <span class="identity-title">{p.title}</span>
          <ChatMarks muted={p.chat.muted} header />
        </h2>
        {#if p.chat.kind === 'group'}<p class="identity-note">
            {p.chat.members.length} members
          </p>{/if}
      </div>
      <AvatarBadge size="md" name={p.title} {avatar} />
    </button>
  </div>
  <div class="header-actions">
    {#if p.chat.kind === 'direct'}
      <button
        class="call-button"
        type="button"
        aria-label={`Call ${p.title}`}
        title={`Call ${p.title}`}
        onclick={p.onCall}
      >
        <ToolbarIcon name="phone" />
      </button>
    {/if}
    <button
      class="menu-button"
      type="button"
      aria-label={t('Chat menu')}
      title={t('Chat menu')}
      onclick={(event) => {
        const box = event.currentTarget.getBoundingClientRect();
        p.onMenu(box.right - 220, box.bottom + 6);
      }}
    >
      <ToolbarIcon name="kebab" />
    </button>
  </div>
  <PinBar pins={p.pins} canUnpin={p.chat.canPin} onJump={p.onJump} onUnpin={p.onUnpin} />
</header>
