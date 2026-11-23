<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  import '#lib/components/styles/links.css';
  import '#lib/components/styles/ux/linkcomposer.css';

  /**
   * web's link composer (in DetailLinkRequestsSection) in the create flows' look (Owner Notes
   * Mockups §5): first find the project or event, then say why, then members of both vote.
   */
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

  let step = $state(0);
  let query = $state('');
  let results = $state<Candidate[]>([]);
  let searching = $state(false);
  let searchError = $state('');
  let selected = $state<Candidate | null>(null);
  let summary = $state('');
  let handle: ReturnType<typeof setTimeout> | undefined;

  const ready = $derived(step === 0 ? !!selected : !!summary.trim() && !pending);

  function schedule(next: string): void {
    query = next;
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
      searchError = t('Could not search right now.');
      results = [];
    } finally {
      searching = false;
    }
  }

  function next(): void {
    if (!ready) return;
    if (step === 0) step = 1;
    else if (selected) onSubmit(selected.id, summary.trim());
  }
</script>

<OverlaySheet
  open
  title={t('Link a project')}
  labelledById="link-composer-title"
  wide
  rootClass="create-sheet create-main link-composer-sheet"
  {onClose}
>
  {#snippet headerActions()}
    <button class="create-close" type="button" aria-label={t('Close')} onclick={onClose}>×</button>
  {/snippet}
  {#snippet subtitle()}
    <p class="sheet-description step-count">
      {t('Step {n} of {total}', { n: step + 1, total: 2 })}
    </p>
  {/snippet}
  {#snippet toolbar()}
    <div class="step-line" aria-hidden="true"><i class="on"></i><i class:on={step === 1}></i></div>
  {/snippet}
  {#if step === 0}
    <h3 class="step-question">{t('Which project or event?')}</h3>
    <label class="link-find">
      <svg aria-hidden="true" viewBox="0 0 24 24" width="16" height="16"
        ><circle cx="11" cy="11" r="7" fill="none" stroke="currentColor" stroke-width="2" /><path
          d="M20 20l-4-4"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
        /></svg
      >
      <input
        value={query}
        aria-label={t('Search by name')}
        placeholder={t('Search projects and events')}
        type="search"
        oninput={(e) => schedule(e.currentTarget.value)}
      />
    </label>
    {#if searching}
      <p class="link-status">{t('Searching…')}</p>
    {:else if searchError}
      <p class="link-status">{searchError}</p>
    {:else if query.trim().length >= 2 && results.length === 0}
      <p class="link-status">{t('No matching projects or events.')}</p>
    {:else if results.length > 0}
      <div class="link-results" role="radiogroup" aria-label={t('Results')}>
        {#each results as item (item.id)}
          <button
            class="link-result"
            class:picked={selected?.id === item.id}
            type="button"
            role="radio"
            aria-checked={selected?.id === item.id}
            onclick={() => (selected = item)}
          >
            <strong>{item.title}</strong>
            <span class="link-radio" aria-hidden="true"></span>
          </button>
        {/each}
      </div>
    {:else}
      <p class="link-status">{t('Type at least two letters of its name.')}</p>
    {/if}
  {:else}
    <h3 class="step-question">{t('Why should they be linked?')}</h3>
    {#if selected}
      <div class="link-picked">
        <small>{t('Linking with')}</small>
        <strong>{selected.title}</strong>
      </div>
    {/if}
    <textarea
      class="link-why"
      bind:value={summary}
      aria-label={t('Why are these linked?')}
      placeholder={t('Shared work, follow-on maintenance, related event, etc.')}
      rows="5"></textarea>
    <p class="link-status">{t('Members of both vote on it.')}</p>
    {#if message}<p class="link-status error" role="alert">{message}</p>{/if}
  {/if}
  {#snippet footer()}
    <div class="sheet-actions">
      {#if step === 1}
        <button class="sheet-cancel create-back" type="button" onclick={() => (step = 0)}
          >{t('Back')}</button
        >
      {/if}
      <button class="sheet-submit" class:faded={!ready} type="button" onclick={next}>
        {step === 0 ? t('Continue') : pending ? t('Working...') : t('Propose link')}
      </button>
    </div>
  {/snippet}
</OverlaySheet>
