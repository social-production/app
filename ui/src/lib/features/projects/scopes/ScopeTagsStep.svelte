<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { scopeTag, tagScope, type Scope, type ScopeKind } from '#lib/domain/scopes.ts';
  import { listScopes } from '#lib/services/scopes.ts';

  /**
   * The create flows' channel and community step (ADR 0068; Polish Pass mockups, 2026-11-16):
   * one search finds both, ticking a row adds it, and the chosen ones sit above as chips. Tags
   * are `#channel` / `@community`; the first is the main one ("primary tag"), except for an
   * event, which has none. A proposal's main tag must be a channel, so its first channel
   * always leads.
   */
  type Variant = 'project' | 'event' | 'thread' | 'help';

  interface Props {
    tags: string[];
    onChange: (tags: string[]) => void;
    variant?: Variant;
  }

  let { tags, onChange, variant = 'project' }: Props = $props();

  let query = $state('');
  let results = $state<Scope[]>([]);
  let searching = $state(false);
  let asked = 0;

  const kindOf = (t: string): ScopeKind | undefined => tagScope(t)?.kind;
  const nameOf = (t: string) => t.slice(1);
  const hasMain = $derived(variant !== 'event');

  const HINTS: Record<Variant, string> = {
    project: 'Pick at least one channel. The first channel is its main one.',
    thread: 'Pick at least one. The first is where it mainly lives.',
    help: 'Pick at least one. Requests are tagged like threads; the first is the main one.',
    event: 'Pick the channels people follow, and any communities.',
  };

  $effect(() => {
    const q = query;
    const mine = ++asked;
    searching = true;
    const timer = setTimeout(() => {
      void listScopes(q, 12)
        .then((found) => {
          if (mine === asked) results = found;
        })
        .catch(() => mine === asked && (results = []))
        .finally(() => mine === asked && (searching = false));
    }, 250);
    return () => clearTimeout(timer);
  });

  function toggle(scope: Scope): void {
    const tag = scopeTag(scope.kind, scope.slug);
    if (tags.includes(tag)) return onChange(tags.filter((t) => t !== tag));
    // A proposal's first channel leads, so its main tag is always a channel.
    const leads =
      variant === 'project' &&
      scope.kind === 'channel' &&
      !tags.some((t) => kindOf(t) === 'channel');
    onChange(leads ? [tag, ...tags] : [...tags, tag]);
  }

  const remove = (tag: string) => onChange(tags.filter((t) => t !== tag));
</script>

<div class="tag-step">
  <label class="tag-search">
    <svg aria-hidden="true" viewBox="0 0 24 24"
      ><circle cx="11" cy="11" r="7" /><path d="m20 20-3.5-3.5" /></svg
    >
    <input
      bind:value={query}
      aria-label={t('Search channels and communities')}
      placeholder={t('Search channels and communities')}
      autocomplete="off"
    />
  </label>

  {#if tags.length > 0}
    <ul class="tag-chips" aria-label={t('Chosen')}>
      {#each tags as tag, i (tag)}
        <li class="tag-chip">
          <span>{kindOf(tag) === 'community' ? '' : '#'}{nameOf(tag)}</span>
          {#if hasMain && i === 0}<small>{t('main')}</small>{/if}
          <button type="button" aria-label={`Remove ${nameOf(tag)}`} onclick={() => remove(tag)}
            >×</button
          >
        </li>
      {/each}
    </ul>
  {/if}

  <p class="tag-hint">{HINTS[variant]}</p>

  <div
    class="tag-results"
    role="listbox"
    aria-multiselectable="true"
    aria-label={t('Channels and communities')}
  >
    {#each results as scope (scope.id)}
      {@const on = tags.includes(scopeTag(scope.kind, scope.slug))}
      <button
        type="button"
        role="option"
        aria-selected={on}
        aria-label={scope.handle}
        class="tag-result"
        class:on
        onclick={() => toggle(scope)}
      >
        <span class="tag-glyph" class:community={scope.kind === 'community'}
          >{scope.kind === 'community' ? scope.handle.charAt(0).toUpperCase() : '#'}</span
        >
        <span class="tag-copy">
          <strong>{scope.handle}</strong>
          <small
            >{scope.kind === 'community' ? t('Community') : t('Channel')} · {scope.members}
            {scope.members === 1 ? 'member' : 'members'}</small
          >
        </span>
        <span class="tag-tick" aria-hidden="true"
          >{#if on}✓{/if}</span
        >
      </button>
    {:else}
      <p class="tag-empty">
        {searching ? t('Searching…') : query ? `Nothing called "${query}" yet.` : ''}
      </p>
    {/each}
  </div>
</div>
