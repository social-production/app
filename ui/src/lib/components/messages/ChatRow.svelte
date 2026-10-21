<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';

  import ChatMarks from './ChatMarks.svelte';

  /**
   * One row of web's Messages list: avatar, name, time, last message and unread count, with
   * pin and mute marks. Right-click or a long press (450 ms) opens the chat menu, as on web.
   */
  interface Props {
    title: string;
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

  let { title, preview, time, unread, pinned, muted, kind, onOpen, onMenu }: Props = $props();

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
  <AvatarBadge size="sm" name={title} />
  <div class="conversation-copy">
    <div class="conversation-topline">
      <ChatMarks {pinned} />
      <strong>{title}</strong>
      <ChatMarks {muted} />
      {#if kind}<span class="chat-kind">{kind}</span>{/if}
      <span class="conversation-time">{time}</span>
    </div>
    <p class="conversation-preview">{preview}</p>
  </div>
  {#if unread > 0}<span class="unread-pill">{unread}</span>{/if}
</button>
