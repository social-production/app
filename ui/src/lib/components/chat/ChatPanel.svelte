<script lang="ts">
  import { tick } from 'svelte';

  import ConfirmDialog from '#lib/components/overlay/ConfirmDialog.svelte';
  import PhotoViewer from '#lib/components/overlay/PhotoViewer.svelte';
  import type { ChatAttachment, ChatLine } from '#lib/domain/chat.ts';
  import type { Mentionable } from '#lib/domain/mentions.ts';

  import BubbleMenu from './BubbleMenu.svelte';
  import ChatBubble from './ChatBubble.svelte';
  import ChatComposer from './ChatComposer.svelte';
  import ChatNote from './ChatNote.svelte';
  import { messageDayKey, messageDayLabel, showsAuthor } from './chatFormat.ts';
  import { BubbleMenuState } from './bubbleMenuState.svelte.ts';
  import { centerIn, scrollToEnd, syncPanelHeight } from './chatViewport.ts';

  import '#lib/components/styles/chat.css';

  /**
   * web's LiveChatPanel for text chats: day dividers, author runs, reply quotes, the bubble
   * menu (profile, reply, copy, pin, edit and delete your own), "left the group" and call lines and
   * the composer, with @mentions where web has them, and photos and files in direct and
   * group chats (`fileUrl`, ADR 0049). Reports come later.
   */
  interface Props {
    lines: ChatLine[];
    title: string;
    placeholder: string;
    submitLabel: string;
    emptyCopy: string;
    embedded: boolean;
    fitViewport: boolean;
    paused: boolean;
    error: string;
    /** web's `showHeader`: the Messages page has its own chat header. */
    showHeader?: boolean;
    /** A direct chat names nobody above bubbles: there is only the other person. */
    hideAuthors?: boolean;
    onSend: (body: string, replyToId: string | null, files: File[]) => Promise<void>;
    /** Set where web's `allowAttachments` is: an attachment's bytes as a URL. */
    fileUrl?: ((attachment: ChatAttachment) => Promise<string>) | undefined;
    onDelete: (id: string) => Promise<void>;
    /** web's conversationActions: edit your own messages (direct and group chats). */
    onEdit?: (id: string, body: string) => Promise<void>;
    /** Set when the viewer may pin here (web's onTogglePin). */
    onTogglePin?: ((id: string, pinned: boolean) => Promise<void>) | null;
    /** A message to scroll to and flash (web's highlightedMessageId, from the pin bar). */
    focus?: { id: string; n: number } | null;
    /** web's linked comment (`?comment=`): scrolled to and marked until the viewer writes. */
    highlighted?: string | null;
    /** Called after the viewer sends, so the link's mark can go (web's clearHighlighted…). */
    onWrote?: () => void;
    mentionSearch?: ((query: string) => Promise<Mentionable[]>) | null;
    mentionAgent?: ((name: string) => string | null) | undefined;
    /** Set for guests: the composer's button is web's "Sign in" (LiveChatPanel's promptChatSignIn). */
    signIn?: (() => void) | null;
  }

  let p: Props = $props();

  let panel = $state<HTMLElement>();
  let log = $state<HTMLDivElement>();
  let keyboardOpen = $state(false);
  let replyTarget = $state<ChatLine | null>(null);
  let editing = $state<ChatLine | null>(null);
  const menu = new BubbleMenuState();
  let confirming = $state<ChatLine | null>(null);
  let copyError = $state('');
  let flashed = $state<string | null>(null);
  let flashTimer: ReturnType<typeof setTimeout> | undefined;
  let photo = $state<{ url: string; name: string; from: HTMLElement | null } | null>(null);
  // Bubbles by message id, for reply jumps (not reactive state).
  const elements: Record<string, HTMLElement | undefined> = {};

  function sync(): void {
    if (panel) keyboardOpen = syncPanelHeight(panel, p);
  }

  const scrollToBottom = (): void => scrollToEnd(log);

  // A new message (anyone's) scrolls the log to the bottom, as on web; a poll that brings
  // nothing new leaves the reader where they are. A linked message is shown first instead.
  const count = $derived(p.lines.length);
  let shownLink: string | null = null;
  $effect(() => {
    void count;
    const link = p.highlighted;
    void tick().then(() => {
      if (!link) return scrollToBottom();
      if (shownLink === link) return;
      const target = elements[link];
      if (!target && count === 0) return;
      shownLink = link;
      if (target && log) centerIn(log, target, 'auto');
      else scrollToBottom();
    });
  });

  $effect(() => {
    void p.embedded;
    void p.fitViewport;
    void tick().then(sync);
    const viewport = window.visualViewport;
    viewport?.addEventListener('resize', sync);
    viewport?.addEventListener('scroll', sync);
    return () => {
      viewport?.removeEventListener('resize', sync);
      viewport?.removeEventListener('scroll', sync);
    };
  });

  function register(node: HTMLElement, id: string): { destroy: () => void } {
    elements[id] = node;
    return { destroy: () => (elements[id] = undefined) };
  }

  /** Tapping a reply quote jumps to the original and flashes it. */
  function jump(id: string | null): void {
    const target = id ? elements[id] : undefined;
    if (!id || !target || !log) return;
    centerIn(log, target, 'smooth');
    flashed = id;
    clearTimeout(flashTimer);
    flashTimer = setTimeout(() => (flashed = null), 1800);
  }

  $effect(() => {
    if (p.focus) jump(p.focus.id);
  });

  function reply(message: ChatLine): void {
    editing = null;
    replyTarget = message;
    menu.close();
  }

  function beginEdit(message: ChatLine): void {
    replyTarget = null;
    editing = message;
    menu.close();
  }

  async function pin(message: ChatLine): Promise<void> {
    menu.close();
    await p.onTogglePin?.(message.id, message.pinned);
  }

  async function copy(message: ChatLine): Promise<void> {
    menu.close();
    try {
      await navigator.clipboard.writeText(message.body);
    } catch {
      copyError = 'Could not copy that message';
    }
  }

  async function send(body: string, files: File[]): Promise<void> {
    const edited = editing;
    if (edited && p.onEdit) {
      await p.onEdit(edited.id, body);
      editing = null;
      return;
    }
    const quoted = replyTarget;
    replyTarget = null;
    copyError = '';
    try {
      await p.onSend(body, quoted?.id ?? null, files);
    } catch (e) {
      replyTarget = quoted;
      throw e;
    }
    p.onWrote?.();
    await tick();
    scrollToBottom();
  }

  async function settleDelete(confirmed: boolean): Promise<void> {
    const message = confirming;
    confirming = null;
    if (confirmed && message) await p.onDelete(message.id);
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === 'Escape') menu.close();
  }}
  onclick={menu.clicked}
  onpointerdown={menu.pointerDown}
  onresize={sync}
/>

<section
  bind:this={panel}
  class:embedded={p.embedded}
  class:fit-viewport={p.fitViewport && !p.embedded}
  class:keyboard-open={keyboardOpen}
  class:headerless={!(p.showHeader ?? true)}
  class="chat-panel message-variant"
>
  {#if p.showHeader ?? true}
    <div class="chat-header">
      <div><h2>{p.title}</h2></div>
    </div>
  {/if}

  <div bind:this={log} class="chat-log">
    <div class="chat-log-stack">
      {#if p.lines.length === 0}
        <div class="empty-state">{p.emptyCopy}</div>
      {:else}
        {#each p.lines as message, i (message.id)}
          {@const previous = i > 0 ? p.lines[i - 1] : undefined}
          {#if !previous || messageDayKey(previous.createdAt) !== messageDayKey(message.createdAt)}
            <div class="day-divider" role="separator">
              <span>{messageDayLabel(message.createdAt)}</span>
            </div>
          {/if}
          {#if message.kind !== 'text'}
            <ChatNote {message} />
          {:else}
            <ChatBubble
              {message}
              showAuthor={!p.hideAuthors && showsAuthor(message, previous)}
              flashed={flashed === message.id}
              highlighted={p.highlighted === message.id}
              mentionAgent={p.mentionAgent}
              canReply={!p.paused && !message.pending}
              onOpenMenu={menu.show}
              onReply={reply}
              onJump={jump}
              {register}
              fileUrl={p.fileUrl}
              onViewPhoto={(url, name, event) =>
                (photo = { url, name, from: event.currentTarget as HTMLElement | null })}
            />
          {/if}
        {/each}
      {/if}
    </div>
  </div>

  {#if menu.open}
    <BubbleMenu
      message={menu.open.message}
      x={menu.open.x}
      y={menu.open.y}
      canWrite={!p.paused && !p.signIn}
      canPin={!!p.onTogglePin}
      canEdit={!!p.onEdit}
      onReply={reply}
      onCopy={copy}
      onPin={pin}
      onEdit={beginEdit}
      onDelete={(m) => ((confirming = m), menu.close())}
      onClose={menu.close}
    />
  {/if}

  <ChatComposer
    placeholder={p.placeholder}
    submitLabel={p.submitLabel}
    {replyTarget}
    {editing}
    paused={p.paused}
    error={p.error || copyError}
    onSubmit={send}
    onCancelReply={() => (replyTarget = null)}
    onCancelEdit={() => (editing = null)}
    onFocus={sync}
    mentionSearch={p.mentionSearch}
    attachments={!!p.fileUrl && !p.signIn}
    signIn={p.signIn}
  />
</section>

<PhotoViewer
  url={photo?.url ?? null}
  alt={photo?.name ?? 'Photo'}
  onClose={() => {
    photo?.from?.focus();
    photo = null;
  }}
/>

<ConfirmDialog
  open={confirming !== null}
  title="Delete this message?"
  body="It is removed for everyone."
  confirmLabel="Delete"
  danger
  onSettle={(ok) => void settleDelete(ok)}
/>
