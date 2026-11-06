<script lang="ts">
  import type { Snippet } from 'svelte';

  import { dateLine } from '#lib/domain/activityTime.ts';
  import type { ServiceRequest } from '#lib/domain/serviceRequests.ts';
  import { relativeTime } from '#lib/domain/time.ts';

  import '#lib/components/styles/service-requests.css';

  /** web's CollapsibleServiceRequestCard: title, time, status badge, then the details. */
  interface Props {
    request: ServiceRequest;
    requester: string;
    highlighted?: boolean;
    children?: Snippet;
  }

  let { request, requester, highlighted = false, children }: Props = $props();

  // svelte-ignore state_referenced_locally
  let open = $state(highlighted);
  $effect(() => {
    if (highlighted) open = true;
  });

  const over = $derived(!!request.endsAt && request.endsAt / 1000 < Date.now());
  const label = $derived(
    request.status === 'accepted'
      ? over
        ? 'Done'
        : 'Accepted'
      : request.status === 'declined'
        ? 'Declined'
        : request.status === 'cancelled'
          ? 'Cancelled'
          : 'Pending',
  );
  const tone = $derived(
    request.status === 'accepted' ? 'complete' : request.status === 'open' ? 'upcoming' : 'locked',
  );
  const when = $derived(
    request.startsAt && request.endsAt
      ? dateLine(request.startsAt, request.endsAt)
      : 'No time requested',
  );
</script>

<details
  id={`request-${request.id}`}
  bind:open
  class:expanded={open}
  class:highlighted
  class="request-card-shell"
>
  <summary class="collapse-toggle">
    <div class="request-header">
      <div class="request-copy">
        <strong>{request.title}</strong>
        <span>{when}</span>
      </div>
      <span class={`phase-badge ${tone}`}>{label}</span>
    </div>
    <div class="request-footer">
      <span class="request-meta">Requested by {requester} · {relativeTime(request.createdAt)}</span>
    </div>
  </summary>

  {#if open}
    <div class="request-body">
      <p>{request.body}</p>
      <!-- web's "It's over. Mark it done and rate it…" hint comes with bookings (ADR 0076 b). -->
      {#if request.status === 'declined' && request.declineReason}
        <p class="decline-reason">Reason: {request.declineReason}</p>
      {/if}
      {@render children?.()}
    </div>
  {/if}
</details>
