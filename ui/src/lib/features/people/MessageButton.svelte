<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { resolve } from '$app/paths';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import type { Contact } from '#lib/domain/messages.ts';

  /** A profile's "Message" action: straight to the chat with this person, as Telegram does
   * (empty until the first message is sent; your existing chat if you have one). */
  let { person }: { person: Contact } = $props();
</script>

<a
  aria-label={`Message ${person.name}`}
  class="action"
  title={t('Message')}
  href={resolve(`/messages?to=${encodeURIComponent(person.agent)}` as '/messages')}
>
  <ToolbarIcon name="message" />
  <span class="action-text">{t('Message')}</span>
</a>
