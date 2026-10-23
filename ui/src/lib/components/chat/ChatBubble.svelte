<script lang="ts">
  import { resolve } from '$app/paths';
  import type { ChatAttachment, ChatLine } from '#lib/domain/chat.ts';

  import { bodyParts, formatByteSize, formatMessageTime } from './chatFormat.ts';

  /**
   * One message of web's LiveChatPanel (text, photos and files, reply quote, time, read
   * receipt, hover reply). Reports arrive with their milestone.
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
    /** Direct and group chats: an attachment's bytes as a URL (ADR 0049). */
    fileUrl?: ((attachment: ChatAttachment) => Promise<string>) | undefined;
    onViewPhoto?: ((url: string, name: string, event: MouseEvent) => void) | undefined;
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
    fileUrl,
    onViewPhoto,
  }: Props = $props();

  const photos = $derived(message.attachments.filter((a) => a.kind === 'image'));
  const files = $derived(message.attachments.filter((a) => a.kind !== 'image'));
  const caption = $derived(Boolean(message.body.trim()));

  /** Shows the photo once its parts arrive (a copy being sent has them locally). */
  function photoSrc(node: HTMLImageElement, a: ChatAttachment): void {
    void fileUrl?.(a).then(
      (url) => (node.src = url),
      () => undefined,
    );
  }

  async function viewPhoto(a: ChatAttachment, event: MouseEvent): Promise<void> {
    event.stopPropagation();
    const url = await fileUrl?.(a).catch(() => null);
    if (url) onViewPhoto?.(url, a.name, event);
  }

  /** web's file bubble is a download link; here the bytes are read first. */
  async function download(a: ChatAttachment, event: MouseEvent): Promise<void> {
    event.preventDefault();
    event.stopPropagation();
    const url = await fileUrl?.(a).catch(() => null);
    if (!url) return;
    const link = document.createElement('a');
    link.href = url;
    link.download = a.name;
    link.click();
  }
</script>

{#snippet stamp()}
  <span class="message-time"
    >{message.pending ? 'Sending…' : formatMessageTime(message.createdAt)}</span
  >
  {#if message.receipt}
    {@const read = message.receipt === 'read'}
    <span class="receipt" class:read aria-label={read ? 'Read' : 'Delivered'}
      >{read ? '✓✓' : '✓'}</span
    >
  {/if}
  {#if message.edited}<span class="edited-mark">edited</span>{/if}
{/snippet}

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
  <div
    class="message-copy"
    class:captionless={photos.length > 0 && !caption}
    class:has-caption={photos.length > 0 && caption}
    class:photo-stack={photos.length > 0}
    class:multi-photo={photos.length > 1}
  >
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
    {#if photos.length > 0}
      <div class="photo-row">
        {#each photos as photo, i (i)}
          <button
            class="photo-button"
            type="button"
            aria-label={`View ${photo.name}`}
            onclick={(event) => void viewPhoto(photo, event)}
          >
            <img class="message-photo" alt={photo.name} use:photoSrc={photo} />
          </button>
        {/each}
      </div>
    {/if}
    {#if files.length > 0}
      <div class="file-row">
        {#each files as file, i (i)}
          <div class="message-file-bubble">
            <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- a download -->
            <a
              class="message-file"
              href="#download"
              onclick={(event) => void download(file, event)}
            >
              <span class="message-file-name">{file.name}</span>
              <span class="message-file-size">{formatByteSize(file.size)}</span>
            </a>
            {#if !caption && photos.length === 0 && i === files.length - 1}
              <span class="message-footer">{@render stamp()}</span>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
    {#if caption}
      <div class="caption-block" class:caption-bubble={photos.length > 0}>
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
          {#if photos.length === 0}<span class="message-footer">{@render stamp()}</span>{/if}
        </p>
      </div>
    {/if}
    {#if photos.length > 0 || (!caption && files.length === 0)}
      <div class="message-footer photo-footer">{@render stamp()}</div>
    {/if}
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
