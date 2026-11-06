<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import ServiceRequestCard from '#lib/components/requests/ServiceRequestCard.svelte';
  import { isPending } from '#lib/domain/serviceRequests.ts';

  import { resolve } from '$app/paths';

  import type { ServiceRequestsState } from './serviceRequestsState.svelte.ts';

  /** web's personal-requests-sheet: Requests (the creator) or Your requests (anyone else). */
  interface Props {
    state: ServiceRequestsState;
    reviewer: boolean;
    requestsOff: boolean;
    highlighted: string | null;
  }

  let { state, reviewer, requestsOff, highlighted }: Props = $props();

  const requests = $derived(state.shown);
  const chatHref = (seed: string): string =>
    `${resolve('/messages')}?conversation=${encodeURIComponent(seed)}`;
  const title = $derived(reviewer ? 'Requests' : 'Your requests');
</script>

<OverlaySheet
  open={state.listOpen}
  {title}
  labelledById="personal-requests-sheet"
  onClose={() => (state.listOpen = false)}
>
  <section class="service-ui card-rail-section">
    {#if state.error}<p class="validation-copy" role="alert">{state.error}</p>{/if}
    {#if !state.loaded}
      <div class="empty-card">Loading requests…</div>
    {:else if requests.length === 0}
      <div class="empty-card">
        {#if requestsOff}
          Not taking requests.
        {:else if reviewer}
          No requests yet.
        {:else}
          You have not requested this service yet.
        {/if}
      </div>
    {:else}
      <div class="card-rail">
        {#each requests as request (request.id)}
          <div id={`request-card-${request.id}`} class="rail-card">
            <ServiceRequestCard
              {request}
              requester={state.names[request.requester] ?? 'Someone'}
              highlighted={highlighted === request.id}
            >
              {#if isPending(request.status) && reviewer}
                {#if state.declining === request.id}
                  <textarea
                    bind:value={state.declineReason}
                    rows="2"
                    maxlength="500"
                    placeholder="Reason (optional)"></textarea>
                  <div class="binary-row review-actions">
                    <button class="vote-chip" type="button" onclick={() => (state.declining = '')}>
                      Back
                    </button>
                    <button
                      class="vote-chip negative"
                      type="button"
                      disabled={state.busy}
                      onclick={() => void state.decide(request, 'declined', state.declineReason)}
                    >
                      Decline request
                    </button>
                  </div>
                {:else}
                  <div class="binary-row review-actions">
                    <button
                      class="vote-chip"
                      type="button"
                      disabled={state.busy}
                      onclick={() => void state.decide(request, 'accepted')}
                    >
                      Accept
                    </button>
                    <button
                      class="vote-chip negative"
                      type="button"
                      onclick={() => {
                        state.declining = request.id;
                        state.declineReason = '';
                      }}
                    >
                      Decline
                    </button>
                  </div>
                {/if}
              {:else if isPending(request.status)}
                <div class="binary-row review-actions">
                  <button
                    class="vote-chip negative"
                    type="button"
                    disabled={state.busy}
                    onclick={() => void state.decide(request, 'cancelled')}
                  >
                    Cancel request
                  </button>
                </div>
              {/if}
              <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved here -->
              <a class="conversation-link" href={chatHref(request.chat)}>Open conversation</a>
            </ServiceRequestCard>
          </div>
        {/each}
      </div>
    {/if}
  </section>
</OverlaySheet>
