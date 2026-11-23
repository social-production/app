<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';

  import '#lib/components/styles/ux/dockmore.css';

  /**
   * The joined dock's ⋯ (Detail Pages Cleanup §4): Edit (when you may propose), Members,
   * Copy link and Leave, in the same menu card as chats. Leaving asks first (the page's
   * own confirmation).
   */
  interface Props {
    noun: string;
    canEdit: boolean;
    onEdit: () => void;
    onMembers: () => void;
    onLeave: () => void;
  }

  let { noun, canEdit, onEdit, onMembers, onLeave }: Props = $props();

  let open = $state(false);
  let copied = $state(false);
  let root = $state<HTMLSpanElement>();

  // A tap anywhere else on the page closes it. A fixed backdrop can't do this: the dock's
  // blur makes it the backdrop's containing block, so it only covered the dock and shifted it.
  function outside(event: PointerEvent): void {
    if (open && root && !root.contains(event.target as Node)) open = false;
  }

  function pick(action: () => void): void {
    open = false;
    action();
  }

  async function copy(): Promise<void> {
    await navigator.clipboard?.writeText(`${location.origin}${location.pathname}`).catch(() => {});
    copied = true;
    setTimeout(() => {
      copied = false;
      open = false;
    }, 900);
  }
</script>

<svelte:document onpointerdown={outside} />
<svelte:window onkeydown={(e) => open && e.key === 'Escape' && (open = false)} />

<span class="dock-more" bind:this={root}>
  <button
    type="button"
    class="dock-icon"
    aria-label={t('More actions')}
    title={t('More')}
    aria-haspopup="menu"
    aria-expanded={open}
    onclick={() => (open = !open)}
  >
    <ToolbarIcon name="more" />
  </button>
  {#if open}
    <div class="dock-more-card" role="menu">
      {#if canEdit}
        <button type="button" role="menuitem" onclick={() => pick(onEdit)}>
          <ToolbarIcon name="pencil" />{t('Edit')}
        </button>
      {/if}
      <button type="button" role="menuitem" onclick={() => pick(onMembers)}>
        <ToolbarIcon name="people" />{t('Members')}
      </button>
      <button type="button" role="menuitem" onclick={() => void copy()}>
        <ToolbarIcon name="link" />{copied ? t('Link copied') : t('Copy link')}
      </button>
      <hr />
      <button type="button" role="menuitem" class="danger" onclick={() => pick(onLeave)}>
        <ToolbarIcon name="leave" />Leave {noun}
      </button>
    </div>
  {/if}
</span>
