<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { portal } from '#lib/components/actions/portal.ts';

  import '#lib/components/styles/links.css';

  /** web's link composer (in DetailLinkRequestsSection): find a project, say why, start the vote. */
  interface Candidate {
    id: string;
    title: string;
  }

  interface Props {
    pending: boolean;
    message: string | undefined;
    search: (query: string) => Promise<Candidate[]>;
    onSubmit: (targetId: string, summary: string) => void;
    onClose: () => void;
  }

  let { pending, message, search, onSubmit, onClose }: Props = $props();

  let query = $state('');
  let results = $state<Candidate[]>([]);
  let searching = $state(false);
  let searchError = $state('');
  let selected = $state<Candidate | null>(null);
  let summary = $state('');
  let handle: ReturnType<typeof setTimeout> | undefined;

  function schedule(next: string): void {
    query = next;
    selected = null;
    clearTimeout(handle);
    const trimmed = next.trim();
    if (trimmed.length < 2) {
      results = [];
      searching = false;
      searchError = '';
      return;
    }
    searching = true;
    handle = setTimeout(() => void run(trimmed), 220);
  }

  async function run(q: string): Promise<void> {
    searchError = '';
    try {
      results = await search(q);
    } catch {
      searchError = 'Could not search right now.';
      results = [];
    } finally {
      searching = false;
    }
  }

  function choose(item: Candidate): void {
    selected = item;
    query = item.title;
    results = [];
  }
</script>

<div class="link-composer" use:portal>
  <div
    class="composer-backdrop"
    onclick={onClose}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
    role="presentation"
    tabindex="-1"
  >
    <div
      aria-modal="true"
      aria-labelledby="link-composer-title"
      class="composer-modal"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      tabindex="-1"
    >
      <div class="composer-head">
        <div class="heading-copy">
          <h3 id="link-composer-title">{t('Propose a new link')}</h3>
          <p>{t('Find the project or event by name, then say why they should be linked.')}</p>
        </div>
        <button class="text-action" type="button" onclick={onClose}>{t('Cancel')}</button>
      </div>
      <label class="field-stack">
        <span class="field-label">{t('Search by name')}</span>
        <input
          value={query}
          placeholder={t('Start typing a project or event name')}
          type="search"
          oninput={(e) => schedule(e.currentTarget.value)}
        />
      </label>
      {#if selected}
        <div class="selected-chip">
          <strong>{selected.title}</strong>
          <button class="text-action" type="button" onclick={() => (selected = null)}
            >{t('Clear')}</button
          >
        </div>
      {:else if searching}
        <div class="search-status">{t('Searching…')}</div>
      {:else if searchError}
        <div class="search-status">{searchError}</div>
      {:else if query.trim().length >= 2 && results.length === 0}
        <div class="search-status">{t('No matching projects or events.')}</div>
      {:else if results.length > 0}
        <div class="search-results">
          {#each results as item (item.id)}
            <button class="search-result" type="button" onclick={() => choose(item)}>
              <strong>{item.title}</strong>
              <span class="row-meta">{t('project')}</span>
            </button>
          {/each}
        </div>
      {/if}
      <label class="field-stack">
        <span class="field-label">{t('Why are these linked?')}</span>
        <textarea
          bind:value={summary}
          placeholder={t('Shared work, follow-on maintenance, related event, etc.')}
          rows="4"></textarea>
      </label>
      {#if message}<p class="search-status" role="alert">{message}</p>{/if}
      <div class="composer-actions">
        <button
          class="primary-button"
          disabled={pending || !selected || !summary.trim()}
          type="button"
          onclick={() => selected && onSubmit(selected.id, summary.trim())}
          >{pending ? t('Working...') : t('Create link vote')}</button
        >
      </div>
    </div>
  </div>
</div>
