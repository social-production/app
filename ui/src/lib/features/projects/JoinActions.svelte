<script lang="ts">
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { resolve } from '$app/paths';

  import DetailShare from './DetailShare.svelte';
  import type { DetailState } from './detailState.svelte.ts';
  import GuestShare from './GuestShare.svelte';

  /**
   * The dock before you join (Detail Pages Cleanup §4): signed out, "Sign up to join" (it
   * opens sign-up) and Share; signed in, Join and Share. The member count lives in Details.
   */
  let { detail, share = true }: { detail: DetailState; share?: boolean } = $props();

  const guest = $derived(connection.current.status === 'signed-out');
  const title = $derived(detail.project?.charter?.title ?? '');
</script>

{#if guest}
  <a class="dock-labelled primary" href={resolve('/onboarding')}>Sign up to join</a>
  {#if share}<GuestShare {title} />{/if}
{:else}
  <button
    id="participation-join"
    type="button"
    class="dock-labelled primary"
    disabled={detail.joining}
    onclick={() => detail.requestToggleJoin()}
  >
    <ToolbarIcon name="person-plus" />Join
  </button>
  {#if share}<DetailShare {detail} />{/if}
{/if}
