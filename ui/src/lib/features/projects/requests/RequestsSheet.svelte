<script lang="ts">
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import ServiceRequestCard from '#lib/components/requests/ServiceRequestCard.svelte';
  import { isPending, notMet, type ServiceRequest } from '#lib/domain/serviceRequests.ts';

  import { resolve } from '$app/paths';

  import type { ServiceRequestsState } from './serviceRequestsState.svelte.ts';

  /** web's personal-requests-sheet: Requests (the creator) or Your requests (anyone else).
   * A collective service's members plan a request into an activity instead of accepting it
   * (web's CollectiveServicePhaseFive "Plan request", ADR 0089). */
  interface Props {
    state: ServiceRequestsState;
    reviewer: boolean;
    requestsOff: boolean;
    highlighted: string | null;
    onPlan?: ((request: ServiceRequest) => void) | undefined;
    /** A collective service's (ADR 0113): no Accept or Decline, a request stays open until
     * planned, and its conversation is the planners' chat (null until you are in one). */
    chatOf?: ((request: ServiceRequest) => string | null) | undefined;
  }

  let { state, reviewer, requestsOff, highlighted, onPlan, chatOf }: Props = $props();
  const collective = $derived(!!chatOf);
  const conversation = (r: ServiceRequest): string | null => (chatOf ? chatOf(r) : r.chat);

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
      <SkeletonRows count={3} kind="row" />
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
              {#if collective && reviewer}
                {#if isPending(request.status) && !notMet(request) && onPlan}
                  <div class="binary-row review-actions">
                    <button class="vote-chip" type="button" onclick={() => onPlan(request)}>
                      Plan request
                    </button>
                  </div>
                {/if}
              {:else if isPending(request.status) && reviewer}
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
                    {#if onPlan}
                      <button class="vote-chip" type="button" onclick={() => onPlan(request)}>
                        Plan request
                      </button>
                    {:else}
                      <button
                        class="vote-chip"
                        type="button"
                        disabled={state.busy}
                        onclick={() => void state.decide(request, 'accepted')}
                      >
                        Accept
                      </button>
                    {/if}
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
              {@const seed = conversation(request)}
              {#if seed}
                <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved here -->
                <a class="conversation-link" href={chatHref(seed)}>Open conversation</a>
              {/if}
            </ServiceRequestCard>
          </div>
        {/each}
      </div>
    {/if}
  </section>
</OverlaySheet>
