<script lang="ts">
  import { resolve } from '$app/paths';
  import type { ChatLine } from '#lib/domain/chat.ts';

  import { openReport } from './chatReports.ts';

  /**
   * web LiveChatPanel's bubble menu: profile, reply, copy, pin, edit and delete your own, and
   * in a space's chat Report and the open report's votes (ADR 0071 part b).
   */
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
    /** Set in every text chat (web's `subjectId`; direct and group chats since ADR 0075). */
    onReport?: ((m: ChatLine) => void) | undefined;
    onVote?: ((m: ChatLine, report: string, yes: boolean) => void) | undefined;
    onClose: () => void;
  }

  let p: Props = $props();
  const m = $derived(p.message);
  const sent = $derived(!m.pending);
  const report = $derived(openReport(m.moderation));
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
  {#if !m.own && sent && p.onReport}
    <button type="button" role="menuitem" onclick={() => p.onReport?.(m)}>Report</button>
  {/if}
  {#if report && p.onVote}
    <button type="button" role="menuitem" onclick={() => p.onVote?.(m, report, true)}
      >Vote to remove</button
    >
    <button type="button" role="menuitem" onclick={() => p.onVote?.(m, report, false)}
      >Vote to keep</button
    >
  {/if}
</div>
