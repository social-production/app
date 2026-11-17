<script lang="ts">
  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import { checkHandle, type ScopeKind } from '#lib/domain/scopes.ts';
  import { explain } from '#lib/services/errors.ts';
  import { createScope } from '#lib/services/scopes.ts';

  import { ScopeArt } from './scopeArt.svelte.ts';
  import ScopeSheetArt from './ScopeSheetArt.svelte';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  /**
   * web's CreateChannelPage and CreateCommunityPage: a handle (its URL) and a description
   * (ADR 0068). A community's Openness "Private" gets its own network, run by its members or
   * by organizers (web's "Who decides"; part d).
   */
  let { kind }: { kind: ScopeKind } = $props();

  const channel = $derived(kind === 'channel');
  const noun = $derived(channel ? 'Channel' : 'Community');
  let name = $state('');
  let description = $state('');
  let openness = $state<'open' | 'invite_only'>('open');
  let governance = $state<'collaborative' | 'organizer'>('collaborative');
  let message = $state('');
  let submitting = $state(false);
  // The creator may add a picture and cover now, with no vote (ADR 0109), or leave the default.
  const art = new ScopeArt();
  const check = $derived(checkHandle(name, `${noun} name`, kind));
  const path = (slug: string) =>
    channel ? resolve('/channels/[slug]', { slug }) : resolve('/communities/[slug]', { slug });
  const missing = $derived(
    [!check.ok && `${noun} handle`, !description.trim() && 'Description'].filter(
      (m): m is string => !!m,
    ),
  );

  async function create(): Promise<void> {
    if (!check.ok || submitting) return;
    submitting = true;
    message = '';
    try {
      const closed =
        !channel && openness === 'invite_only'
          ? { organized: governance === 'organizer' }
          : undefined;
      const slug = await createScope(kind, name, description, closed, {
        picture: art.newPicture?.hash ?? null,
        cover: art.newCover?.hash ?? null,
      });
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
    <ScopeSheetArt {kind} {art} canEdit />
    <label>
      <RequiredLabel>{noun} handle</RequiredLabel>
      <span class="prefixed">
        {#if channel}<span class="prefix" aria-hidden="true">#</span>{/if}
        <input
          bind:value={name}
          aria-required="true"
          placeholder={channel ? 'local-gardening' : 'neighborhood-group'}
        />
      </span>
    </label>
    {#if !name.trim()}
      <p class="helper-text">Lowercase, with dashes. This is its address.</p>
    {:else}
      {#if !check.ok}
        <p class="status-note">{check.error}</p>
      {:else}
        <p class="helper-text">Its address: <code>{path(check.slug)}</code></p>
      {/if}
    {/if}
    {#if !channel}
      <label>
        <span class="field-label">Openness</span>
        <select bind:value={openness}>
          <option value="open">Open</option>
          <option value="invite_only">Private</option>
        </select>
      </label>

      {#if openness === 'invite_only'}
        <label>
          <span class="field-label">Who decides</span>
          <select bind:value={governance}>
            <option value="collaborative">Collaborative: all members vote on changes</option>
            <option value="organizer">Organizer controlled: you and organizers you choose</option>
          </select>
          <span class="helper-text">
            <!-- web's copy, less what isn't ported (renames, switching; ADR 0068). -->
            {governance === 'organizer'
              ? 'Organizers edit settings and remove members directly. This stays as it is.'
              : 'Members vote on the description, picture and cover, and on removing members. This stays as it is.'}
          </span>
        </label>
      {/if}
    {/if}
    <label>
      <RequiredLabel>Description</RequiredLabel>
      <textarea bind:value={description} rows="4" aria-required="true"></textarea>
    </label>
    {#if message}<p class="status-note">{message}</p>{/if}
  </div>
</CreateSheet>
