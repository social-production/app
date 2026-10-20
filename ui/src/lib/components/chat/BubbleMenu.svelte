<script lang="ts">
  import { resolve } from '$app/paths';
  import type { ChatLine } from '#lib/domain/chat.ts';

  /** web LiveChatPanel's bubble menu: profile, reply, copy, pin, edit and delete your own. */
  interface Props {
    message: ChatLine;
    x: number;
    y: number;
    canWrite: boolean;
    canPin: boolean;
    canEdit: boolean;
    onReply: (m: ChatLine) => void;
    onCopy: (m: ChatLine) => void;
    onPin: (m: ChatLine) => void;
    onEdit: (m: ChatLine) => void;
    onDelete: (m: ChatLine) => void;
    onClose: () => void;
  }

  let p: Props = $props();
  const m = $derived(p.message);
  const sent = $derived(!m.pending);
</script>

<div class="bubble-menu" role="menu" style="left: {p.x}px; top: {p.y}px">
  <a role="menuitem" href={resolve('/profile/[agent]', { agent: m.author })} onclick={p.onClose}
    >View profile</a
  >
  {#if p.canWrite && sent}
    <button type="button" role="menuitem" onclick={() => p.onReply(m)}>Reply</button>
  {/if}
  <button type="button" role="menuitem" onclick={() => p.onCopy(m)}>Copy</button>
  {#if p.canPin && sent}
    <button type="button" role="menuitem" onclick={() => p.onPin(m)}
      >{m.pinned ? 'Unpin' : 'Pin'}</button
    >
  {/if}
  {#if p.canEdit && p.canWrite && m.own && sent}
    <button type="button" role="menuitem" onclick={() => p.onEdit(m)}>Edit</button>
  {/if}
  {#if m.own && sent}
    <button type="button" role="menuitem" onclick={() => p.onDelete(m)}>Delete</button>
  {/if}
</div>
