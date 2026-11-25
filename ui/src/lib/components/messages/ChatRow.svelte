<script lang="ts">
  import { withoutRawPaths } from '#lib/components/chat/chatFormat.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import KindGlyph from '#lib/components/icons/KindGlyph.svelte';
  import { kindColor, kindOf } from '#lib/domain/kindGlyphs.ts';
  import type { CharterMode } from '#lib/domain/projects.ts';

  import ChatMarks from './ChatMarks.svelte';

  /**
   * One row of web's Messages list: avatar, name, time, last message and unread count, with
   * pin and mute marks. Right-click or a long press (450 ms) opens the chat menu, as on web.
   */
  interface Props {
    title: string;
    /** A direct chat's other person: their photo. */
    agent?: string | null;
    /** A group's photo. */
    picture?: string | null;
    /** A public chat's space type: its kind tile, as on feed cards, in place of initials. */
    mode?: CharterMode | null;
    preview: string;
    time: string;
    unread: number;
    pinned?: boolean;
    muted?: boolean;
    /** web's chat-kind label on Public rows ("Project"). */
    kind?: string;
    onOpen: () => void;
    onMenu: (x: number, y: number) => void;
  }

  let {
    title,
    agent = null,
    picture = null,
    mode = null,
    preview,
    time,
    unread,
    pinned,
    muted,
    kind,
    onOpen,
    onMenu,
  }: Props = $props();

  let timer: ReturnType<typeof setTimeout> | undefined;
  let origin = { x: 0, y: 0 };
  let held = false;

  function down(event: PointerEvent): void {
    held = false;
    if (event.button !== 0) return;
    origin = { x: event.clientX, y: event.clientY };
    clearTimeout(timer);
    timer = setTimeout(() => {
      timer = undefined;
      held = true;
      onMenu(origin.x, origin.y);
    }, 450);
  }

  function move(event: PointerEvent): void {
    if (timer && Math.hypot(event.clientX - origin.x, event.clientY - origin.y) > 8) {
      clearTimeout(timer);
      timer = undefined;
    }
  }

  function end(): void {
    clearTimeout(timer);
    timer = undefined;
  }

  function click(): void {
    // The click that ends a long press only closes it.
    if (held) {
      held = false;
      return;
    }
    onOpen();
  }
</script>

<button
  class:unread={unread > 0}
  class="conversation-row"
  type="button"
  onclick={click}
  oncontextmenu={(event) => {
    event.preventDefault();
    onMenu(event.clientX, event.clientY);
  }}
  onpointerdown={down}
  onpointermove={move}
  onpointerup={end}
  onpointercancel={end}
>
  {#if mode}
    <span
      class="chat-kind-tile"
      style={`--surface-accent: ${kindColor(kindOf(mode))}`}
      aria-hidden="true"><KindGlyph kind={kindOf(mode)} size={20} /></span
    >
  {:else}
    <AvatarBadge size="sm" name={title} {agent} src={picture} />
  {/if}
  <div class="conversation-copy">
    <div class="conversation-topline">
      <strong>{title}</strong>
      <ChatMarks {muted} />
      {#if kind && !mode}<span class="chat-kind">{kind}</span>{/if}
    </div>
    <p class="conversation-preview">{withoutRawPaths(preview)}</p>
  </div>
  <!-- Telegram's right edge (owner, 2026-11-23): the time on top; under it the unread count,
       grey while muted, or else the pin. -->
  <span class="conversation-meta">
    <span class="conversation-time">{time}</span>
    {#if unread > 0}
      <span class="unread-pill" class:muted>{unread}</span>
    {:else if pinned}
      <ChatMarks {pinned} />
    {/if}
  </span>
</button>
