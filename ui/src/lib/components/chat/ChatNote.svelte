<script lang="ts">
  import type { ChatLine } from '#lib/domain/chat.ts';

  import { callLineText, formatMessageTime } from './chatFormat.ts';

  /** web's centred chat lines: a finished call ("Voice call · 4:12", ADR 0051) or a leave
   * ("<name> left the group."). */
  let { message }: { message: ChatLine } = $props();
</script>

{#if message.kind === 'call'}
  <div
    id={`comment-${message.id}`}
    class="call-line"
    class:missed={message.call?.outcome !== 'completed'}
  >
    <span aria-hidden="true">📞</span>
    <span>{callLineText(message)}</span>
    <span class="message-time">{formatMessageTime(message.createdAt)}</span>
  </div>
{:else}
  <div id={`comment-${message.id}`} class="call-line system-line">
    <span>{message.authorName} left the group.</span>
    <span class="message-time">{formatMessageTime(message.createdAt)}</span>
  </div>
{/if}
