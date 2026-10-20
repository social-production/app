<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import ComposeMessageSheet from '#lib/components/messages/ComposeMessageSheet.svelte';
  import type { Contact, NewChat } from '#lib/domain/messages.ts';
  import { explain } from '#lib/services/errors.ts';
  import { contacts, sendNewChat } from '#lib/services/messages.ts';

  /** web's profile "Message" action: the compose sheet with this person chosen, then the chat. */
  interface Props {
    person: Contact;
    viewer: string;
  }

  let { person, viewer }: Props = $props();
  let open = $state(false);
  let people: Promise<Contact[]> | undefined;

  async function search(query: string): Promise<Contact[]> {
    people ??= contacts();
    const needle = query.trim().toLowerCase();
    return (await people)
      .filter((c) => c.agent !== viewer && c.name.toLowerCase().includes(needle))
      .slice(0, 8);
  }

  async function send(chat: NewChat): Promise<void> {
    let id: string;
    try {
      id = await sendNewChat(chat);
    } catch (e) {
      throw new Error(explain(e), { cause: e });
    }
    open = false;
    await goto(resolve(`/messages?conversation=${encodeURIComponent(id)}` as '/messages'));
  }
</script>

<button
  aria-label={`Message ${person.name}`}
  class="action"
  title="Message"
  type="button"
  onclick={() => (open = true)}
>
  <ToolbarIcon name="message" />
  <span class="action-text">Message</span>
</button>

<ComposeMessageSheet
  {open}
  prefill={person}
  {search}
  onSend={send}
  onClose={() => (open = false)}
/>
