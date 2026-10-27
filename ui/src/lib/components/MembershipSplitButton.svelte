<script lang="ts">
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';

  import '#lib/components/styles/detail.css';

  /** web's MembershipSplitButton: Join/Joined on the left, the member count (opens the list) on the right. */
  interface Props {
    joined: boolean;
    count: number;
    pending?: boolean;
    membersOpen?: boolean;
    /** web's `joinAriaLabel`: "Join event" on event pages. */
    noun?: 'project' | 'event';
    onToggleJoin: () => void;
    onOpenMembers: () => void;
  }

  let {
    joined,
    count,
    pending = false,
    membersOpen = false,
    noun = 'project',
    onToggleJoin,
    onOpenMembers,
  }: Props = $props();

  const joinAria = $derived(joined ? `Leave ${noun}` : `Join ${noun}`);
</script>

<div class="membership-split" class:joined class:pending class:members-open={membersOpen}>
  <button
    id="participation-join"
    aria-label={joinAria}
    title={joinAria}
    aria-pressed={joined}
    class="membership-join"
    class:joined
    class:pending
    disabled={pending}
    type="button"
    onclick={onToggleJoin}
  >
    <ToolbarIcon name={joined ? 'person-check' : 'person-plus'} />
    <span class="join-text">{joined ? 'Joined' : 'Join'}</span>
  </button>
  <button
    aria-expanded={membersOpen}
    aria-label={`${count} members`}
    title={`${count} members`}
    class="membership-count"
    type="button"
    onclick={onOpenMembers}>{count}</button
  >
</div>
