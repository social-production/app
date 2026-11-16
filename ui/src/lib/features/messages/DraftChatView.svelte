<script lang="ts">
  import ChatPanel from '#lib/components/chat/ChatPanel.svelte';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import type { Contact } from '#lib/domain/messages.ts';

  /**
   * A chat with someone you have not written to yet (Telegram's empty chat): the normal
   * header and message bar over "No messages here yet". The conversation is created by the
   * first message, so opening this writes nothing.
   */
  interface Props {
    person: Contact;
    /** Starts the chat with this first message; the page then opens it. */
    onFirst: (body: string, files: File[]) => Promise<void>;
    onBack: () => void;
  }

  let { person, onFirst, onBack }: Props = $props();

  let error = $state('');
  let sending = $state(false);

  async function send(body: string, _replyTo: string | null, files: File[]): Promise<void> {
    if (sending) return;
    sending = true;
    error = '';
    try {
      await onFirst(body.trim(), files);
    } catch (e) {
      error = e instanceof Error ? e.message : 'The message could not be sent.';
      throw e;
    } finally {
      sending = false;
    }
  }
</script>

<header class="chat-header">
  <button
    class="back-button"
    type="button"
    aria-label="Back to chats"
    title="Back to chats"
    onclick={onBack}
  >
    <svg aria-hidden="true" viewBox="0 0 24 24" width="22" height="22">
      <path
        d="M15 5l-7 7 7 7"
        fill="none"
        stroke="currentColor"
        stroke-width="2.4"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>
  <div class="chat-identity">
    <div class="identity-trigger">
      <div><h2><span class="identity-title">{person.name}</span></h2></div>
      <AvatarBadge size="md" name={person.name} />
    </div>
  </div>
</header>

<ChatPanel
  lines={[]}
  title={person.name}
  placeholder="Write a message..."
  submitLabel="Send"
  emptyCopy={sending ? 'Sending…' : 'No messages here yet.'}
  embedded={true}
  fitViewport={false}
  showHeader={false}
  hideAuthors={true}
  paused={false}
  {error}
  onSend={send}
  fileUrl={() => Promise.reject(new Error('Not sent yet.'))}
  onDelete={() => Promise.resolve()}
/>
