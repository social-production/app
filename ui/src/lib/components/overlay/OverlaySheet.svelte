<script lang="ts">
  import type { Snippet } from 'svelte';

  import { fitText } from '#lib/components/actions/fitText.ts';
  import { portal } from '#lib/components/actions/portal.ts';

  import '#lib/components/styles/sheets.css';

  /**
   * web's OverlaySheet, the base of every popup: a bottom sheet on phones, a centred panel
   * on desktop. Close is always the footer × (a "Close" bar when there are no actions); a
   * footer "Cancel" button is hidden as a duplicate of it.
   */
  interface Props {
    open: boolean;
    title: string;
    labelledById?: string;
    wide?: boolean;
    /** web's activity sheet (wider body, its own close) and elevated layer (above a sheet). */
    activity?: boolean;
    elevated?: boolean;
    /** Extra class on the root, so a sheet's own ported styles can scope under it. */
    rootClass?: string;
    onClose: () => void;
    subtitle?: Snippet;
    toolbar?: Snippet;
    footer?: Snippet;
    children: Snippet;
  }

  let {
    open,
    title,
    labelledById = 'overlay-sheet-title',
    wide = false,
    activity = false,
    elevated = false,
    rootClass = '',
    onClose,
    subtitle,
    toolbar,
    footer,
    children,
  }: Props = $props();

  let footerEl = $state<HTMLElement>();
  let closeOnly = $state(true);

  const isCancel = (el: Element): boolean =>
    (el.textContent ?? '').replace(/\s+/g, ' ').trim().toLowerCase() === 'cancel';

  function syncFooter(): void {
    if (!footerEl) return;
    const controls = [...footerEl.querySelectorAll('button:not(.sheet-close), a')];
    for (const el of controls)
      el.classList.toggle(
        'sheet-dismiss-duplicate',
        el instanceof HTMLButtonElement && isCancel(el),
      );
    closeOnly = controls.every((el) => el.classList.contains('sheet-dismiss-duplicate'));
  }

  $effect(() => {
    if (!footerEl) return;
    const observer = new MutationObserver(syncFooter);
    // Text changes too: wizards relabel "Cancel" to "Back" in place.
    observer.observe(footerEl, { childList: true, subtree: true, characterData: true });
    syncFooter();
    return () => observer.disconnect();
  });

  // Lock page scroll while open, keeping the scrollbar's width so nothing shifts.
  $effect(() => {
    if (!open) return;
    const gap = window.innerWidth - document.documentElement.clientWidth;
    document.body.style.overflow = 'hidden';
    if (gap > 0) document.body.style.paddingRight = `${gap}px`;
    const escape = (e: KeyboardEvent): void => {
      if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
      }
    };
    window.addEventListener('keydown', escape);
    return () => {
      document.body.style.overflow = '';
      document.body.style.paddingRight = '';
      window.removeEventListener('keydown', escape);
    };
  });
</script>

{#if open}
  <div class={`overlay-root ${rootClass}`} class:elevated role="presentation" use:portal>
    <button aria-label="Close" title="Close" class="overlay-scrim" type="button" onclick={onClose}
    ></button>
    <div
      aria-labelledby={labelledById}
      aria-modal="true"
      class="overlay-sheet has-footer"
      class:has-toolbar={!!toolbar}
      class:wide
      class:activity
      role="dialog"
    >
      <header class="overlay-header">
        <div class="overlay-header-copy">
          <h2 id={labelledById} use:fitText={{ min: 13, text: title }}>{title}</h2>
          {@render subtitle?.()}
        </div>
      </header>
      {#if toolbar}<div class="overlay-toolbar">{@render toolbar()}</div>{/if}
      <div class="overlay-body">{@render children()}</div>
      <footer
        bind:this={footerEl}
        class="overlay-footer inline-close"
        class:has-actions={!!footer}
        class:close-only={closeOnly}
      >
        <button aria-label="Close" class="overlay-close sheet-close" type="button" onclick={onClose}
          >×{#if closeOnly}<span class="close-label">Close</span>{/if}</button
        >
        {@render footer?.()}
      </footer>
    </div>
  </div>
{/if}
