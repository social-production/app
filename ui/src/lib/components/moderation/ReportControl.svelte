<script lang="ts">
  import {
    VISIBLE,
    type Moderation,
    type ModerationState,
    type ReportReason,
  } from '#lib/domain/moderation.ts';

  import ReportComposer from './ReportComposer.svelte';
  import ReportMenu from './ReportMenu.svelte';
  import { reportActions } from './reportActions.ts';

  import '#lib/components/styles/report.css';

  /**
   * web's ReportControl (ADR 0071): the ⋮ menu on a card or a detail header, and the report
   * composer. Cards pass the index's state; the open report itself loads when the menu opens.
   */
  interface Props {
    /** The space's listing id. */
    id: string;
    /** One of the space's chat messages or comments, when that is what is reported. */
    message?: string;
    itemLabel: string;
    /** The state the card or page already knows (`visible` when nothing is reported). */
    known?: ModerationState;
    /** What a detail page already read (wins over `known`). */
    moderation?: Moderation | null;
    /** The item's author: they can't report it (web's blockedMessage). */
    owner?: string;
    /** web's copyUrl: the page link for "Copy link" (empty leaves it out). */
    copyUrl?: string;
    /** Told when a report or vote changes the state (a page re-reads or hides the item). */
    onChange?: (next: Moderation) => void;
  }

  let {
    id,
    message,
    itemLabel,
    known = 'visible',
    moderation = null,
    owner = '',
    copyUrl = '',
    onChange,
  }: Props = $props();

  const actions = reportActions();
  let menuOpen = $state(false);
  let composerOpen = $state(false);
  let pending = $state(false);
  let loading = $state(false);
  let loaded = $state<Moderation | null>(null);
  let reason = $state<ReportReason>('spam');
  let description = $state('');
  let error = $state('');
  let voteError = $state('');

  const current = $derived(loaded ?? moderation ?? { ...VISIBLE, state: known });

  /** A parent told of changes keeps the state (and re-reads it); otherwise this menu does. */
  function apply(next: Moderation): void {
    if (onChange) onChange(next);
    else loaded = next;
  }

  async function load(): Promise<void> {
    if (loaded || moderation || known === 'visible') return;
    loading = true;
    try {
      loaded = await actions.load(id, message);
    } catch {
      // The menu still offers Copy link and Report; a report already open refuses a second.
    } finally {
      loading = false;
    }
  }

  function toggle(): void {
    if (!actions.requireViewer(`Sign in to report this ${itemLabel}.`)) return;
    menuOpen = !menuOpen;
    voteError = '';
    if (menuOpen) void load();
  }

  async function copy(): Promise<boolean> {
    try {
      await navigator.clipboard.writeText(copyUrl);
      return true;
    } catch {
      return false;
    }
  }

  function closeComposer(): void {
    composerOpen = false;
    reason = 'spam';
    description = '';
    error = '';
  }

  async function submit(): Promise<void> {
    pending = true;
    error = '';
    try {
      apply(await actions.report(id, reason, description, message));
      closeComposer();
    } catch (e) {
      error = actions.explain(e);
    } finally {
      pending = false;
    }
  }

  async function vote(yes: boolean): Promise<void> {
    const report = current.report;
    if (!report) return;
    menuOpen = false;
    pending = true;
    voteError = '';
    try {
      apply(await actions.vote(id, report.id, yes));
    } catch (e) {
      // The menu reopens with the reason (web toasts it).
      voteError = actions.explain(e);
      menuOpen = true;
    } finally {
      pending = false;
    }
  }
</script>

<div class="report-control">
  <ReportMenu
    {itemLabel}
    moderation={current.state}
    report={current.report}
    open={menuOpen}
    {pending}
    {loading}
    error={voteError}
    {copyUrl}
    blocked={!!owner && owner === actions.me()}
    onToggle={toggle}
    onClose={() => (menuOpen = false)}
    onCopy={copy}
    onCompose={() => {
      menuOpen = false;
      composerOpen = true;
    }}
    onVote={(yes) => void vote(yes)}
  />
</div>

<ReportComposer
  open={composerOpen}
  {itemLabel}
  {pending}
  {error}
  bind:reason
  bind:description
  onClose={closeComposer}
  onSubmit={() => void submit()}
/>
