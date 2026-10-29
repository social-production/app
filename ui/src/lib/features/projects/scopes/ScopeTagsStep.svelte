<script lang="ts">
  import ScopeTagSelector from '#lib/components/create/ScopeTagSelector.svelte';
  import { scopeTag, tagScope, type ScopeKind } from '#lib/domain/scopes.ts';
  import { listScopes } from '#lib/services/scopes.ts';

  /**
   * web's create-flow scope step: a primary channel tag, then (on request) more channels and
   * communities (ADR 0068). Tags are `#channel` / `@community`; the first channel leads.
   */
  interface Props {
    tags: string[];
    onChange: (tags: string[]) => void;
    helperText?: string;
  }

  let { tags, onChange, helperText = 'At least one channel tag is required.' }: Props = $props();

  let showMore = $state(false);
  const channels = $derived(tags.filter((t) => tagScope(t)?.kind === 'channel'));
  const communities = $derived(tags.filter((t) => tagScope(t)?.kind === 'community'));
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
  <ScopeTagSelector
    label="Primary channel tag"
    placeholder="Type to search channels"
    {helperText}
    selected={channels.slice(0, 1).map(item)}
    search={search('channel')}
    onAdd={(t) => add(t, true)}
    onRemove={remove}
  />
  <label class="scope-more-toggle">
    <input bind:checked={showMore} type="checkbox" />
    <span>Add more channels or communities</span>
  </label>
  {#if showMore}
    <ScopeTagSelector
      label="Additional channel tags"
      placeholder="Optional"
      selected={channels.slice(1).map(item)}
      search={search('channel')}
      onAdd={(t) => add(t)}
      onRemove={remove}
    />
    <ScopeTagSelector
      label="Community tags"
      placeholder="Optional"
      selected={communities.map(item)}
      search={search('community')}
      onAdd={(t) => add(t)}
      onRemove={remove}
    />
  {/if}
</div>
