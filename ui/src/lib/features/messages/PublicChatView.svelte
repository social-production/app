<script lang="ts">
  import { t, plural } from '#lib/i18n/index.ts';
  import { untrack } from 'svelte';

  import ChatPanel from '#lib/components/chat/ChatPanel.svelte';
  import { ChatState } from '#lib/components/chat/chatState.svelte.ts';
  import { MentionSource } from '#lib/components/chat/mentionSource.svelte.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import ChatMarks from '#lib/components/messages/ChatMarks.svelte';
  import PeopleSheet from '#lib/components/overlay/PeopleSheet.svelte';
  import type { PublicChat } from '#lib/domain/messages.ts';
  import type { Person } from '#lib/domain/people.ts';
  import { projectAttachmentBlob } from '#lib/services/attachments.ts';
  import { deleteMessage, listMessages, postMessage } from '#lib/services/chat.ts';
  import { explain } from '#lib/services/errors.ts';
  import { mentionPool } from '#lib/services/messages.ts';
  import { notifyMentions } from '#lib/services/notices.ts';
  import { namesOf, viewer } from '#lib/services/people.ts';
  import { replyNotice } from '#lib/services/pushAsks.ts';
  import { markProjectChatRead, projectChatMembers } from '#lib/services/publicChats.ts';

  import { spaceHref } from '#lib/components/cards/spaceHref.ts';
  import type { ChatSummary, ListPrefs } from '#lib/domain/messages.ts';
  import { projectPageOf } from '#lib/services/notifications.ts';

  import ChatAboutPage from './ChatAboutPage.svelte';
  import { inTurn } from './writes.ts';

  /** web's MessagesPage with a project chat open from the Public tab: its header ("Project
   * chat · N members"), then the same chat as the project's Chat tab, read as it arrives. */
  interface Props {
    chat: PublicChat;
    onBack: () => void;
    onRead: () => void;
    /** Pin or mute from the chat's page. */
    onPrefs: (prefs: ListPrefs) => void;
  }

  let { chat, onBack, onRead, onPrefs }: Props = $props();

  // The page keys this view by chat, so the first id is the one to keep.
  // svelte-ignore state_referenced_locally
  const id = chat.id;
  const project = { seed: id };
  const talk = new ChatState(
    {
      list: () => listMessages(project),
      post: async (body, replyTo, files) => {
        const sent = await inTurn(() => postMessage(project, body, replyTo, files));
        const link = `${location.pathname}${location.search}`;
        const answered = replyTo ? talk.messages.find((m) => m.id === replyTo)?.author : null;
        void replyNotice(project, answered ?? null, body, link);
        notifyMentions(mentions.mentioned(body), 'in a project chat', body, link);
        return sent;
      },
      file: (attachment) => projectAttachmentBlob(project, attachment),
      remove: (messageId) => inTurn(() => deleteMessage(project, messageId)),
      viewer,
      names: namesOf,
      explain,
    },
    {},
  );

  // People who wrote here come first in @mention suggestions (ADR 0047).
  const mentions = new MentionSource(
    () => mentionPool(),
    () => ({ me: talk.me, agents: Object.keys(talk.names), names: talk.names }),
  );

  $effect(() => mentions.watch(talk.messages.map((m) => m.body)));

  const members = $derived(`${plural(chat.members, '{n} member', '{n} members')}`);

  // web's members panel: the count opens the project's members, read when asked for.
  let membersOpen = $state(false);
  let people = $state<Person[]>([]);
  let membersRequest = 0;

  async function toggleMembers() {
    if (membersOpen) {
      membersOpen = false;
      return;
    }
    const request = ++membersRequest;
    try {
      const found = await projectChatMembers(id);
      if (request !== membersRequest) return;
      people = found;
      membersOpen = true;
    } catch (cause) {
      if (request === membersRequest) talk.error = explain(cause);
    }
  }

  // The chat's page (Owner Notes Mockups §7): tap the name, as for a person.
  let about = $state(false);
  let pageHref = $state<string | null>(null);
  let memberAgents = $state<string[]>([]);

  async function openAbout(): Promise<void> {
    about = true;
    const [found, listing] = await Promise.all([
      projectChatMembers(id).catch(() => [] as Person[]),
      projectPageOf(id).catch(() => null),
    ]);
    memberAgents = found.map((person) => person.agent);
    for (const person of found) talk.names[person.agent] ??= person.profile?.name ?? 'Unnamed';
    pageHref = listing ? spaceHref(chat.mode, listing) : null;
  }

  const summary = $derived<ChatSummary>({
    id,
    kind: 'group',
    creator: '',
    manager: null,
    managers: [],
    picture: null,
    members: memberAgents,
    title: chat.title,
    last: null,
    unread: 0,
    invitedAt: 0,
    member: true,
    canPin: true,
    pinned: chat.pinned,
    muted: chat.muted,
    hidden: false,
  });

  // untrack: starting reads chat state, which must not re-run (and so stop) this effect.
  $effect(() => {
    untrack(() => talk.start(() => true));
    return () => talk.dispose();
  });

  $effect(() => {
    const newest = talk.messages.at(-1)?.createdAt;
    if (newest) void markProjectChatRead(id, newest).then(onRead);
  });
</script>

<header class="chat-header">
  <button
    class="back-button"
    type="button"
    aria-label={t('Back to chats')}
    title={t('Back to chats')}
    onclick={onBack}
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
    <div class="identity-trigger">
      <div>
        <button class="title-hit" type="button" onclick={() => void openAbout()}>
          <h2>
            <span class="identity-title">{chat.title}</span>
            <ChatMarks muted={chat.muted} header />
          </h2>
        </button>
        <p class="identity-note">
          {t('Project chat ·')}
          <button
            aria-expanded={membersOpen}
            class="members-link"
            type="button"
            onclick={toggleMembers}>{members}</button
          >
        </p>
      </div>
      <button
        class="avatar-hit"
        type="button"
        aria-label={`About ${chat.title}`}
        onclick={() => void openAbout()}
      >
        <AvatarBadge size="md" name={chat.title} />
      </button>
    </div>
  </div>
</header>

<ChatPanel
  lines={talk.lines}
  title={chat.title}
  placeholder={t('Message the project...')}
  submitLabel={t('Send')}
  emptyCopy={talk.loaded ? t('No project chat yet.') : ''}
  embedded={true}
  fitViewport={false}
  showHeader={false}
  paused={talk.paused}
  error={talk.error}
  onSend={(body, replyTo, files) => talk.send(body, replyTo, files)}
  fileUrl={talk.fileUrl}
  onDelete={(messageId) => talk.remove(messageId)}
  mentionSearch={mentions.search}
  mentionAgent={mentions.agentOf}
/>

<PeopleSheet
  open={membersOpen}
  title={t('Project members')}
  description={t('Members coordinate planning, updates, and activity together.')}
  emptyCopy={t('No members listed yet.')}
  {people}
  onClose={() => (membersOpen = false)}
/>

{#if about}
  <ChatAboutPage
    chat={summary}
    title={chat.title}
    me={talk.me}
    names={talk.names}
    lines={talk.lines}
    groups={[]}
    fileUrl={talk.fileUrl}
    space={{ mode: chat.mode, href: pageHref }}
    onBack={() => (about = false)}
    onCall={() => undefined}
    {onPrefs}
    onEditGroup={() => undefined}
    onJump={() => (about = false)}
    onOpenChat={() => undefined}
  />
{/if}
