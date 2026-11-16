<script lang="ts">
  import { resolve } from '$app/paths';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
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

{#snippet item(icon: string, label: string, run: () => void, danger = false)}
  <button type="button" role="menuitem" class:menu-danger={danger} onclick={run}
    ><ToolbarIcon name={icon} />{label}</button
  >
{/snippet}

<!-- The floating card of the Chats mockups: an icon on each row, Delete and Report in red
     below a line. -->
<div class="bubble-menu" role="menu" style="left: {p.x}px; top: {p.y}px">
  {#if p.canWrite && sent}{@render item('reply', 'Reply', () => p.onReply(m))}{/if}
  {@render item('copy', 'Copy', () => p.onCopy(m))}
  {#if p.canPin && sent}{@render item('pin', m.pinned ? 'Unpin' : 'Pin', () => p.onPin(m))}{/if}
  {#if p.canEdit && p.canWrite && m.own && sent}{@render item('pencil', 'Edit', () =>
      p.onEdit(m),
    )}{/if}
  {#if !m.own}
    <a role="menuitem" href={resolve('/profile/[agent]', { agent: m.author })} onclick={p.onClose}
      ><ToolbarIcon name="user" />View profile</a
    >
  {/if}
  {#if report && p.onVote}
    {@render item('thumb-down', 'Vote to remove', () => p.onVote?.(m, report, true))}
    {@render item('thumb-up', 'Vote to keep', () => p.onVote?.(m, report, false))}
  {/if}
  {#if sent && (m.own || p.onReport)}
    <hr class="menu-divider" />
    {#if m.own}{@render item('trash', 'Delete', () => p.onDelete(m), true)}{/if}
    {#if !m.own && p.onReport}{@render item('flag', 'Report', () => p.onReport?.(m), true)}{/if}
  {/if}
</div>
