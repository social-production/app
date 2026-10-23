<script lang="ts">
  import { resolve } from '$app/paths';
  import type { ChatLine } from '#lib/domain/chat.ts';

  import { bodyParts, formatMessageTime } from './chatFormat.ts';

  /**
   * One message of web's LiveChatPanel (text, reply quote, time, hover reply). Photos, files,
   * reports and read receipts arrive with their milestones.
   */
  interface Props {
    message: ChatLine;
    showAuthor: boolean;
    flashed: boolean;
    canReply: boolean;
    onOpenMenu: (message: ChatLine, event: MouseEvent) => void;
    onReply: (message: ChatLine) => void;
    onJump: (id: string | null) => void;
    register: (node: HTMLElement, id: string) => { destroy: () => void };
    /** web's linked comment (`?comment=`): marked until the viewer writes. */
    highlighted?: boolean;
    /** Who "@name" means here, for its profile link; `null` when nobody known has that name. */
    mentionAgent?: ((name: string) => string | null) | undefined;
  }

  let {
    message,
    showAuthor,
    flashed,
    canReply,
    onOpenMenu,
    onReply,
    onJump,
    register,
    highlighted = false,
    mentionAgent,
  }: Props = $props();
</script>

<!-- web's bubble opens its menu on click or right-click; the menu's items are buttons. -->
<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
<article
  id={`comment-${message.id}`}
  class:flash={flashed}
  class:highlighted
  class:own={message.own}
  class="chat-message"
  use:register={message.id}
  onclick={(event) => onOpenMenu(message, event)}
  oncontextmenu={(event) => {
    event.preventDefault();
    onOpenMenu(message, event);
  }}
>
  <div class="message-copy">
    {#if showAuthor}
      <a class="author-link" href={resolve('/profile/[agent]', { agent: message.author })}
        >{message.authorName}</a
      >
    {/if}
    {#if message.replyAuthor || message.replyPreview}
      <button
        class="reply-quote"
        type="button"
        title="Show the original message"
        disabled={!message.replyToId}
        onclick={(event) => {
          event.stopPropagation();
          onJump(message.replyToId);
        }}
      >
        <strong>{message.replyAuthor || 'Message'}</strong>
        <span>{message.replyPreview || ''}</span>
      </button>
    {/if}
    <div class="caption-block">
      <p>
        {#each bodyParts(message.body) as part, i (i)}
          {#if part.mention}
            {@const agent = mentionAgent?.(part.mention)}
            {#if agent}
              <a class="mention" href={resolve('/profile/[agent]', { agent })}>{part.text}</a>
            {:else}<span class="mention">{part.text}</span>{/if}
          {:else if part.internal}
            <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- a path written in the message -->
            <a href={part.href}>{part.text}</a>
          {:else if part.href}
            <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- external link -->
            <a href={part.href} target="_blank" rel="noopener noreferrer">{part.text}</a>
          {:else}{part.text}{/if}
        {/each}
        <span class="message-footer">
          <span class="message-time"
            >{message.pending ? 'Sending…' : formatMessageTime(message.createdAt)}</span
          >
          {#if message.edited}<span class="edited-mark">edited</span>{/if}
        </span>
      </p>
    </div>
    {#if canReply && !message.own}
      <button
        class="hover-reply"
        type="button"
        aria-label="Reply"
        title="Reply"
        onclick={(event) => {
          event.stopPropagation();
          onReply(message);
        }}>↩</button
      >
    {/if}
  </div>
</article>
