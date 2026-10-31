<script lang="ts">
  import ScopeTagSelector from '#lib/components/create/ScopeTagSelector.svelte';
  import { scopeTag, tagScope, type ScopeKind } from '#lib/domain/scopes.ts';
  import { listScopes } from '#lib/services/scopes.ts';

  /**
   * web's create-flow scope step: a primary channel tag, then (on request) more channels and
   * communities (ADR 0068). Tags are `#channel` / `@community`; the primary tag leads. Threads
   * (`choosePrimary`, ADR 0070) pick the primary tag's type and always show the extra fields.
   */
  interface Props {
    tags: string[];
    onChange: (tags: string[]) => void;
    helperText?: string;
    choosePrimary?: boolean;
  }

  let {
    tags,
    onChange,
    helperText = 'At least one channel tag is required.',
    choosePrimary = false,
  }: Props = $props();

  let showMore = $state(false);
  let primaryKind = $state<ScopeKind>('channel');
  const kindOf = (t: string) => tagScope(t)?.kind;
  const primary = $derived(tags[0] && kindOf(tags[0]) === primaryKind ? tags[0] : null);
  const rest = $derived(tags.filter((t) => t !== primary));
  const channels = $derived(rest.filter((t) => kindOf(t) === 'channel'));
  const communities = $derived(rest.filter((t) => kindOf(t) === 'community'));
  const item = (tag: string) => ({ key: tag, label: tag.slice(1) });

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
  {#if choosePrimary}
    <p class="helper-text">Choose one primary tag so people can discover this thread.</p>
    <label>
      <span class="field-label">Primary tag type</span>
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
    {helperText}
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
      placeholder="Optional"
      selected={channels.map(item)}
      search={search('channel')}
      onAdd={(t) => add(t)}
      onRemove={remove}
    />
    <ScopeTagSelector
      label={choosePrimary ? 'Additional community tags' : 'Community tags'}
      placeholder="Optional"
      selected={communities.map(item)}
      search={search('community')}
      onAdd={(t) => add(t)}
      onRemove={remove}
    />
  {/if}
</div>
