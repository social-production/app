<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { untrack } from 'svelte';

  import ChatPanel from '#lib/components/chat/ChatPanel.svelte';
  import { ChatModeration } from '#lib/components/chat/chatModeration.svelte.ts';
  import { ChatState } from '#lib/components/chat/chatState.svelte.ts';
  import { withReceipts } from '#lib/components/chat/chatFormat.ts';
  import { MentionSource } from '#lib/components/chat/mentionSource.svelte.ts';
  import GroupInfoSheet from '#lib/components/messages/GroupInfoSheet.svelte';

  import ChatAboutPage from './ChatAboutPage.svelte';
  import ChatHeader from './ChatHeader.svelte';
  import type { ChatPin, ChatSummary } from '#lib/domain/messages.ts';
  import { attachmentBlob } from '#lib/services/attachments.ts';
  import {
    chatModeration,
    reportChatMessage,
    voteOnChatReport,
  } from '#lib/services/chatReports.ts';
  import { explain } from '#lib/services/errors.ts';
  import {
    chatPeerReadAt,
    chatPins,
    deleteChatMessage,
    editChatMessage,
    listChatMessages,
    markChatRead,
    mentionPool,
    pinChatMessage,
    postChatMessage,
  } from '#lib/services/messages.ts';
  import { notifyMentions } from '#lib/services/notices.ts';
  import { namesOf, viewer } from '#lib/services/people.ts';
  import { callRequest } from '#lib/stores/callRequest.svelte.ts';

  import type { InboxState } from './inboxState.svelte.ts';
  import { inTurn } from './writes.ts';

  /** web's MessagesPage with a conversation open: its header (tap it for the chat menu), the
   * pin bar, the chat (polled every 5 s, web's THREAD_POLL_MS, marked read as messages
   * arrive) and a group's "Chat info" sheet. */
  interface Props {
    chat: ChatSummary;
    inbox: InboxState;
    infoOpen: boolean;
    onInfoClose: () => void;
    onMenu: (x: number, y: number) => void;
    onBack: () => void;
    /** The chat's page (tap the face or name), `?about=1`. */
    about: boolean;
    onAbout: (open: boolean) => void;
    onInfo: () => void;
    onOpenChat: (id: string) => void;
  }

  let {
    chat,
    inbox,
    infoOpen,
    onInfoClose,
    onMenu,
    onBack,
    about,
    onAbout,
    onInfo,
    onOpenChat,
  }: Props = $props();

  // The page keys this view by chat, so the first chat and names are the ones to keep.
  // svelte-ignore state_referenced_locally
  const seed = chat.id;
  // svelte-ignore state_referenced_locally
  const thread = new ChatState(
    {
      list: () => listChatMessages(seed),
      post: async (body, replyTo, files) => {
        const id = await inTurn(() => postChatMessage(seed, body, replyTo, files));
        const link = `${location.pathname}?conversation=${encodeURIComponent(seed)}`;
        notifyMentions(mentions.mentioned(body), 'in a chat', body, link);
        return id;
      },
      file: (attachment) => attachmentBlob(seed, attachment),
      remove: (id) => inTurn(() => deleteChatMessage(seed, id)),
      edit: (id, body) => inTurn(() => editChatMessage(seed, id, body)),
      pin: (id, pinned) => inTurn(() => pinChatMessage(seed, id, pinned)),
      viewer,
      names: namesOf,
      explain,
      seen: () =>
        void inTurn(() => markChatRead(seed)).then(
          () => inbox.refresh(),
          () => undefined,
        ),
    },
    inbox.names,
  );

  // Reports on this chat's messages (ADR 0075), read as messages arrive and every 30 s.
  const reports = new ChatModeration({
    read: (ids) => chatModeration(seed, ids),
    report: (message, reason, description) =>
      inTurn(() => reportChatMessage(seed, message, reason, description)),
    vote: (report, yes) => inTurn(() => voteOnChatReport(seed, report, yes)),
  });
  $effect(() => {
    const ids = thread.messages.map((m) => m.id);
    void reports.refresh(ids);
    const timer = setInterval(() => void reports.refresh(ids, true), 30_000);
    return () => clearInterval(timer);
  });

  const title = $derived(inbox.title(chat));
  let pins = $state<ChatPin[]>([]);
  let focus = $state<{ id: string; n: number } | null>(null);
  const members = $derived(
    chat.members.map((agent) => ({ agent, name: inbox.names[agent] ?? '…' })),
  );

  // untrack: starting reads chat state, which must not re-run (and so stop) this effect.
  $effect(() => {
    untrack(() => thread.start(() => true));
    return () => thread.dispose();
  });

  // Pins and the read receipt follow each load of the chat (web reads `peer_read_at` with
  // the thread); receipts are for direct chats only, as on web.
  let peerReadAt = $state<number | null>(null);
  const lines = $derived.by(() => {
    const moderated = thread.lines.map((l) => {
      const moderation = reports.byId[l.id];
      return moderation ? { ...l, moderation } : l;
    });
    return chat.kind === 'direct' ? withReceipts(moderated, peerReadAt) : moderated;
  });
  $effect(() => {
    void thread.messages;
    void chatPins(seed).then(
      (list) => (pins = list),
      () => undefined,
    );
    if (chat.kind === 'direct') {
      void chatPeerReadAt(seed).then(
        (at) => (peerReadAt = at),
        () => undefined,
      );
    }
  });

  // The chat's members come first in @mention suggestions, then your other chats' (ADR 0047).
  const mentions = new MentionSource(
    () => mentionPool(inbox.all),
    () => ({ me: thread.me, agents: chat.members, names: thread.names }),
  );

  $effect(() => mentions.watch(thread.messages.map((m) => m.body)));

  function call(): void {
    callRequest.next = {
      seed,
      name: title,
      peer: chat.members.find((m) => m !== inbox.me) ?? null,
    };
  }

  const partner = $derived(
    chat.kind === 'direct' ? chat.members.find((m) => m !== inbox.me) : null,
  );
  const groups = $derived(
    inbox.all
      .filter((c) => c.kind === 'group' && c.member && !!partner && c.members.includes(partner))
      .map((c) => ({ id: c.id, title: inbox.title(c), count: c.members.length })),
  );

  function jump(id: string): void {
    focus = { id, n: (focus?.n ?? 0) + 1 };
  }
</script>

<ChatHeader
  {chat}
  {title}
  me={inbox.me}
  {pins}
  {onBack}
  onAbout={() => onAbout(true)}
  onCall={call}
  {onMenu}
  onJump={jump}
  onUnpin={(id) => void thread.pin(id, true)}
/>

<ChatPanel
  {lines}
  {title}
  placeholder={t('Write a message...')}
  submitLabel={t('Send')}
  emptyCopy={thread.loaded ? t('No messages yet.') : ''}
  embedded={true}
  fitViewport={false}
  showHeader={false}
  hideAuthors={chat.kind === 'direct'}
  paused={thread.paused}
  error={thread.error}
  {focus}
  onSend={(body, replyTo, files) => thread.send(body, replyTo, files)}
  fileUrl={thread.fileUrl}
  onDelete={(id) => thread.remove(id)}
  onEdit={(id, body) => thread.edit(id, body)}
  onTogglePin={chat.canPin ? (id, pinned) => thread.pin(id, pinned) : null}
  reports={{ report: reports.report, vote: reports.vote, explain, signedIn: () => true }}
  mentionSearch={mentions.search}
  mentionAgent={mentions.agentOf}
/>

{#if about}
  <ChatAboutPage
    {chat}
    {title}
    me={inbox.me}
    names={inbox.names}
    {lines}
    {groups}
    fileUrl={thread.fileUrl}
    onBack={() => onAbout(false)}
    onCall={call}
    onPrefs={(prefs) => void inbox.setPrefs(chat.id, prefs)}
    onEditGroup={onInfo}
    onJump={(id) => {
      onAbout(false);
      jump(id);
    }}
    {onOpenChat}
  />
{/if}

{#if chat.kind === 'group'}
  <GroupInfoSheet
    open={infoOpen}
    {title}
    me={inbox.me}
    manager={chat.manager}
    managers={chat.managers}
    picture={chat.picture}
    {members}
    search={(q) => inbox.search(q)}
    onRename={(name) => inbox.rename(seed, name)}
    onAdd={(c) => inbox.addMember(seed, c.agent)}
    onRemove={(c) => inbox.removeMember(seed, c.agent)}
    onManager={(c, on) => inbox.setManager(seed, c.agent, on)}
    onPicture={(photo) => inbox.setPicture(seed, photo)}
    onClose={onInfoClose}
  />
{/if}
