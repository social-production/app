<script lang="ts">
  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import { checkHandle, type ScopeKind } from '#lib/domain/scopes.ts';
  import { explain } from '#lib/services/errors.ts';
  import { createScope } from '#lib/services/scopes.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  /**
   * web's CreateChannelPage and CreateCommunityPage: a handle (its URL) and a description
   * (ADR 0068). Private communities (web's Openness "Private") come with invites, M5d part (d).
   */
  let { kind }: { kind: ScopeKind } = $props();

  const channel = $derived(kind === 'channel');
  const noun = $derived(channel ? 'Channel' : 'Community');
  let name = $state('');
  let description = $state('');
  let message = $state('');
  let submitting = $state(false);
  const check = $derived(checkHandle(name, `${noun} name`, kind));
  const path = (slug: string) =>
    channel ? resolve('/channels/[slug]', { slug }) : resolve('/communities/[slug]', { slug });
  const missing = $derived(
    [!check.ok && 'handle', !description.trim() && 'description'].filter((m): m is string => !!m),
  );

  async function create(): Promise<void> {
    if (!check.ok || submitting) return;
    submitting = true;
    message = '';
    try {
      const slug = await createScope(kind, name, description);
      // eslint-disable-next-line svelte/no-navigation-without-resolve -- path() resolves it
      await goto(path(slug));
    } catch (e) {
      message = explain(e);
    } finally {
      submitting = false;
    }
  }
</script>

<CreateSheet
  title={`Create ${channel ? 'channel' : 'community'}`}
  description={channel
    ? 'A channel is a topic surface. It gathers related threads and project activity without defining who belongs together.'
    : 'A community is a social space that connects people to projects and discussion, without forcing every topic into one channel.'}
  steps={[{ id: 'basics', title: 'Basics' }]}
  stepIndex={0}
  {missing}
  submitLabel={`Create ${channel ? 'channel' : 'community'}`}
  {submitting}
  onStep={() => {}}
  onSubmit={() => void create()}
  onClose={() => void goto(resolve('/'))}
>
  <div class="form-stack">
    <label>
      <RequiredLabel>{noun} handle</RequiredLabel>
      <input
        bind:value={name}
        aria-required="true"
        placeholder={channel ? 'e.g. local-gardening' : 'e.g. neighborhood-group'}
      />
    </label>
    {#if name.trim()}
      {#if !check.ok}
        <p class="status-note">{check.error}</p>
      {:else}
        <p class="helper-text">URL: <code>{path(check.slug)}</code></p>
      {/if}
    {/if}
    <label>
      <RequiredLabel>Description</RequiredLabel>
      <textarea bind:value={description} rows="4" aria-required="true"></textarea>
    </label>
    {#if message}<p class="status-note">{message}</p>{/if}
  </div>
</CreateSheet>
