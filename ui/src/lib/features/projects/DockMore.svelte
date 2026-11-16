<script lang="ts">
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

<span class="dock-more">
  <button
    type="button"
    class="dock-icon"
    aria-label="More actions"
    title="More"
    aria-haspopup="menu"
    aria-expanded={open}
    onclick={() => (open = !open)}
  >
    <ToolbarIcon name="more" />
  </button>
  {#if open}
    <button
      class="dock-more-backdrop"
      type="button"
      aria-label="Close menu"
      onclick={() => (open = false)}
    ></button>
    <div class="dock-more-card" role="menu">
      {#if canEdit}
        <button type="button" role="menuitem" onclick={() => pick(onEdit)}>
          <ToolbarIcon name="pencil" />Edit
        </button>
      {/if}
      <button type="button" role="menuitem" onclick={() => pick(onMembers)}>
        <ToolbarIcon name="people" />Members
      </button>
      <button type="button" role="menuitem" onclick={() => void copy()}>
        <ToolbarIcon name="link" />{copied ? 'Link copied' : 'Copy link'}
      </button>
      <hr />
      <button type="button" role="menuitem" class="danger" onclick={() => pick(onLeave)}>
        <ToolbarIcon name="leave" />Leave {noun}
      </button>
    </div>
  {/if}
</span>
