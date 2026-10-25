<script lang="ts">
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import type { FollowStatus } from '#lib/domain/follows.ts';

  /** web's profile "Follow" action: Follow → Requested (pending approval) or Following. */
  interface Props {
    name: string;
    status: FollowStatus | null;
    busy: boolean;
    onToggle: () => void;
  }

  let { name, status, busy, onToggle }: Props = $props();

  const label = $derived(
    busy
      ? 'Updating follow'
      : status === 'accepted'
        ? `Unfollow ${name}`
        : status === 'pending'
          ? `Cancel follow request to ${name}`
          : `Follow ${name}`,
  );
</script>

<button
  aria-label={label}
  title={label}
  class="action primary"
  class:on={status !== null}
  disabled={busy}
  type="button"
  onclick={onToggle}
>
  <ToolbarIcon name={status === 'accepted' ? 'person-check' : 'person-plus'} />
  <span class="action-text"
    >{status === 'pending' ? 'Requested' : status === 'accepted' ? 'Following' : 'Follow'}</span
  >
</button>
