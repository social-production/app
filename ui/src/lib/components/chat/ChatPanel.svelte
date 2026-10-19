<script lang="ts">
  import { tick } from 'svelte';

  import { resolve } from '$app/paths';
  import ConfirmDialog from '#lib/components/overlay/ConfirmDialog.svelte';
  import type { ChatLine } from '#lib/domain/chat.ts';

  import ChatBubble from './ChatBubble.svelte';
  import ChatComposer from './ChatComposer.svelte';
  import { messageDayKey, messageDayLabel, showsAuthor } from './chatFormat.ts';
  import { syncPanelHeight } from './chatViewport.ts';

  import '#lib/components/styles/chat.css';

  /**
   * web's LiveChatPanel for text chats: day dividers, author runs, reply quotes, the bubble
   * menu (profile, reply, copy, delete your own) and the composer. Attachments, mentions,
   * reports and pins come with their milestones.
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
    onSend: (body: string, replyToId: string | null) => Promise<void>;
    onDelete: (id: string) => Promise<void>;
  }

  let p: Props = $props();

  let panel = $state<HTMLElement>();
  let log = $state<HTMLDivElement>();
  let keyboardOpen = $state(false);
  let replyTarget = $state<ChatLine | null>(null);
  let menu = $state<{ message: ChatLine; x: number; y: number } | null>(null);
  let blockMenuOpen = false;
  let confirming = $state<ChatLine | null>(null);
  let copyError = $state('');
  let flashed = $state<string | null>(null);
  let flashTimer: ReturnType<typeof setTimeout> | undefined;
  // Bubbles by message id, for reply jumps (not reactive state).
  const elements: Record<string, HTMLElement | undefined> = {};

  function sync(): void {
    if (panel) keyboardOpen = syncPanelHeight(panel, p);
  }

  function scrollToBottom(): void {
    const el = log;
    if (!el) return;
    const apply = (): void => {
      el.scrollTop = el.scrollHeight;
    };
    apply();
    requestAnimationFrame(apply);
  }

  // A new message (anyone's) scrolls the log to the bottom, as on web; a poll that brings
  // nothing new leaves the reader where they are.
  const count = $derived(p.lines.length);
  $effect(() => {
    void count;
    void tick().then(scrollToBottom);
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

  function openMenu(message: ChatLine, event: MouseEvent): void {
    const target = event.target;
    if (target instanceof Element && target.closest('a, button, input, textarea, label')) return;
    if (blockMenuOpen || menu) {
      blockMenuOpen = false;
      menu = null;
      return;
    }
    const [width, height, gap] = [180, 140, 8];
    let x = event.clientX;
    let y = event.clientY + gap;
    const below = window.innerHeight - event.clientY;
    if (below < height + gap && event.clientY > below)
      y = Math.max(8, event.clientY - gap - height);
    if (x + width > window.innerWidth - 8) x = Math.max(8, window.innerWidth - 8 - width);
    menu = { message, x, y };
  }

  function menuPointerDown(event: PointerEvent): void {
    if (!menu) return;
    if (event.target instanceof Element && event.target.closest('.bubble-menu')) return;
    menu = null;
    blockMenuOpen = true;
  }

  /** Tapping a reply quote jumps to the original and flashes it. */
  function jump(id: string | null): void {
    const target = id ? elements[id] : undefined;
    if (!id || !target || !log) return;
    const bounds = log.getBoundingClientRect();
    const box = target.getBoundingClientRect();
    const top = box.top - bounds.top + log.scrollTop - log.clientHeight / 2 + box.height / 2;
    log.scrollTo({ top: Math.max(top, 0), behavior: 'smooth' });
    flashed = id;
    clearTimeout(flashTimer);
    flashTimer = setTimeout(() => (flashed = null), 1800);
  }

  function reply(message: ChatLine): void {
    replyTarget = message;
    menu = null;
  }

  async function copy(message: ChatLine): Promise<void> {
    menu = null;
    try {
      await navigator.clipboard.writeText(message.body);
    } catch {
      copyError = 'Could not copy that message';
    }
  }

  async function send(body: string): Promise<void> {
    const quoted = replyTarget;
    replyTarget = null;
    copyError = '';
    try {
      await p.onSend(body, quoted?.id ?? null);
    } catch (e) {
      replyTarget = quoted;
      throw e;
    }
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
    if (event.key === 'Escape' && menu) menu = null;
  }}
  onclick={() => (blockMenuOpen = false)}
  onpointerdown={menuPointerDown}
  onresize={sync}
/>

<section
  bind:this={panel}
  class:embedded={p.embedded}
  class:fit-viewport={p.fitViewport && !p.embedded}
  class:keyboard-open={keyboardOpen}
  class="chat-panel message-variant"
>
  <div class="chat-header">
    <div><h2>{p.title}</h2></div>
  </div>

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
          <ChatBubble
            {message}
            showAuthor={showsAuthor(message, previous)}
            flashed={flashed === message.id}
            canReply={!p.paused && !message.pending}
            onOpenMenu={openMenu}
            onReply={reply}
            onJump={jump}
            {register}
          />
        {/each}
      {/if}
    </div>
  </div>

  {#if menu}
    {@const target = menu.message}
    <div class="bubble-menu" role="menu" style="left: {menu.x}px; top: {menu.y}px">
      <a
        role="menuitem"
        href={resolve('/profile/[agent]', { agent: target.author })}
        onclick={() => (menu = null)}>View profile</a
      >
      {#if !p.paused && !target.pending}
        <button type="button" role="menuitem" onclick={() => reply(target)}>Reply</button>
      {/if}
      <button type="button" role="menuitem" onclick={() => copy(target)}>Copy</button>
      {#if target.own && !target.pending}
        <button type="button" role="menuitem" onclick={() => ((confirming = target), (menu = null))}
          >Delete</button
        >
      {/if}
    </div>
  {/if}

  <ChatComposer
    placeholder={p.placeholder}
    submitLabel={p.submitLabel}
    {replyTarget}
    paused={p.paused}
    error={p.error || copyError}
    onSubmit={send}
    onCancelReply={() => (replyTarget = null)}
    onFocus={sync}
  />
</section>

<ConfirmDialog
  open={confirming !== null}
  title="Delete this message?"
  body="It is removed for everyone."
  confirmLabel="Delete"
  danger
  onSettle={(ok) => void settleDelete(ok)}
/>
