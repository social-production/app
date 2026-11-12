<script lang="ts">
  import ScopeTagSelector from '#lib/components/create/ScopeTagSelector.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import { scopeTag, tagScope, type ScopeKind } from '#lib/domain/scopes.ts';
  import { listScopes } from '#lib/services/scopes.ts';

  /**
   * web's create-flow scope steps (ADR 0068). Tags are `#channel` / `@community`; a primary
   * tag leads. Each flow keeps web's own layout:
   * - `project`: a primary channel, then (on request) more channels and communities;
   * - `event`: channel tags, then (on request) community tags; no primary tag;
   * - `thread` / `help`: the primary tag's type, then additional channels and communities.
   */
  type Variant = 'project' | 'event' | 'thread' | 'help';

  interface Props {
    tags: string[];
    onChange: (tags: string[]) => void;
    variant?: Variant;
  }

  let { tags, onChange, variant = 'project' }: Props = $props();

  const kindOf = (t: string) => tagScope(t)?.kind;
  const choosePrimary = $derived(variant === 'thread' || variant === 'help');
  let primaryKind = $state<ScopeKind>('channel');
  const primary = $derived(
    variant !== 'event' && tags[0] && kindOf(tags[0]) === primaryKind ? tags[0] : null,
  );
  const rest = $derived(tags.filter((t) => t !== primary));
  const channels = $derived(rest.filter((t) => kindOf(t) === 'channel'));
  const communities = $derived(rest.filter((t) => kindOf(t) === 'community'));
  // web opens the extra fields when they already hold tags (coming back to the step).
  // svelte-ignore state_referenced_locally
  let showMore = $state(variant === 'event' ? communities.length > 0 : rest.length > 0);
  const item = (tag: string) => ({ key: tag, label: tag.slice(1) });

  const COPY = {
    help: {
      primaryHelp: 'Required. Help requests must be tagged like threads.',
      channel: 'Type to add another channel',
      community: 'Type to add another community',
      extraHelp: 'Optional.',
    },
    thread: { primaryHelp: 'Required.', channel: 'Optional', community: 'Optional', extraHelp: '' },
    project: {
      primaryHelp: 'At least one channel tag is required.',
      channel: 'Optional',
      community: 'Optional',
      extraHelp: '',
    },
  };
  const copy = $derived(COPY[variant === 'event' ? 'project' : variant]);

  const search = (kind: ScopeKind) => async (q: string) =>
    (await listScopes(q, 8))
      .filter((s) => s.kind === kind)
      .map((s) => ({ key: scopeTag(kind, s.slug), label: s.handle }));

  function add(tag: string, first = false): void {
    if (tags.includes(tag)) return;
    onChange(first ? [tag, ...tags] : [...tags, tag]);
  }
  const remove = (tag: string) => onChange(tags.filter((t) => t !== tag));
</script>

<div class="form-stack">
  {#if variant === 'event'}
    <ScopeTagSelector
      label="Channel tags"
      placeholder="Type to add a channel tag"
      selected={channels.map(item)}
      search={search('channel')}
      onAdd={(t) => add(t)}
      onRemove={remove}
    />
    <label class="scope-more-toggle">
      <input bind:checked={showMore} type="checkbox" />
      <span>Add communities</span>
    </label>
    {#if showMore}
      <ScopeTagSelector
        label="Community tags"
        placeholder="Type to add a community tag"
        selected={communities.map(item)}
        search={search('community')}
        onAdd={(t) => add(t)}
        onRemove={remove}
      />
    {/if}
  {:else}
    {#if variant === 'thread'}
      <p class="helper-text">Choose one primary tag so people can discover this thread.</p>
    {/if}
    {#if choosePrimary}
      <label>
        {#if variant === 'help'}
          <RequiredLabel>Primary tag type</RequiredLabel>
        {:else}
          <span class="field-label">Primary tag type</span>
        {/if}
        <select bind:value={primaryKind}>
          <option value="channel">Channel</option>
          <option value="community">Community</option>
        </select>
      </label>
    {/if}
    <ScopeTagSelector
      label={primaryKind === 'community' ? 'Primary community tag' : 'Primary channel tag'}
      placeholder={primaryKind === 'community'
        ? 'Type to search communities'
        : 'Type to search channels'}
      helperText={copy.primaryHelp}
      selected={primary ? [item(primary)] : []}
      search={search(primaryKind)}
      onAdd={(t) => add(t, true)}
      onRemove={remove}
    />
    {#if !choosePrimary}
      <label class="scope-more-toggle">
        <input bind:checked={showMore} type="checkbox" />
        <span>Add more channels or communities</span>
      </label>
    {/if}
    {#if showMore || choosePrimary}
      <ScopeTagSelector
        label="Additional channel tags"
        placeholder={copy.channel}
        helperText={copy.extraHelp}
        selected={channels.map(item)}
        search={search('channel')}
        onAdd={(t) => add(t)}
        onRemove={remove}
      />
      <ScopeTagSelector
        label={choosePrimary ? 'Additional community tags' : 'Community tags'}
        placeholder={copy.community}
        helperText={copy.extraHelp}
        selected={communities.map(item)}
        search={search('community')}
        onAdd={(t) => add(t)}
        onRemove={remove}
      />
    {/if}
  {/if}
</div>
