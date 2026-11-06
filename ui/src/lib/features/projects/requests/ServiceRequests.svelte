<script lang="ts">
  import PhaseWorkToolbar from '#lib/components/governance/PhaseWorkToolbar.svelte';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';
  import type { RequestSettings } from '#lib/domain/projects.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import type { DetailState } from '../detailState.svelte.ts';
  import RequestComposer from './RequestComposer.svelte';
  import RequestSettingsSheet from './RequestSettingsSheet.svelte';
  import RequestsSheet from './RequestsSheet.svelte';
  import { ServiceRequestsState } from './serviceRequestsState.svelte.ts';

  import '#lib/components/styles/service-requests.css';

  /**
   * A personal service's requests (web's IndividualServicePhaseOne, ADR 0076 part a): the
   * owner note, "Not taking requests.", and the Requests / Request settings / Request service
   * actions with their sheets. Listed availability and bookings come in part b.
   */
  let { detail, creatorName }: { detail: DetailState; creatorName: string } = $props();

  const me = $derived(connection.current.status === 'connected' ? connection.current.agent : '');
  const project = $derived(detail.project);
  const reviewer = $derived(!!me && project?.creator === me);
  // svelte-ignore state_referenced_locally
  const state = new ServiceRequestsState(
    detail.id,
    () => me,
    () => reviewer,
  );
  const fallback: RequestSettings = { enabled: true, mode: 'both', offSchedule: true };
  const settings = $derived(project?.requests ?? fallback);
  const requestsOff = $derived(!settings.enabled);
  const highlighted = $derived(page.url.searchParams.get('request'));
  const showRequests = $derived(reviewer || state.shown.length > 0);
  const canRequest = $derived(settings.enabled && !!me && !reviewer);

  $effect(() => {
    if (me) void state.load();
  });
  // web opens the Requests sheet on `?request=…` (a notification or chat link).
  $effect(() => {
    if (highlighted) state.listOpen = true;
  });

  async function send(): Promise<void> {
    if (!project) return;
    const title = project.charter?.title ?? 'Service';
    if (await state.send({ title, creator: project.creator, settings })) state.listOpen = true;
  }

  async function saveSettings(): Promise<void> {
    if (await state.saveSettings()) await detail.load();
  }
</script>

<div class="service-ui">
  <p class="owner-note">
    {#if reviewer}
      You run this service. Only you and each person asking can see their request.
    {:else}
      Offered by <a href={resolve('/profile/[agent]', { agent: project?.creator ?? '' })}
        >{creatorName}</a
      >. Requests stay private between you and them.
      {#if !me && !requestsOff}
        <a href={resolve('/onboarding')}>Sign in to request a time</a>.
      {/if}
    {/if}
  </p>
  {#if requestsOff}
    <div class="requests-off-banner" role="status">Not taking requests.</div>
  {/if}
  {#if settings.mode === 'direct'}
    <div class="empty-card helper-card">
      <p>
        This service is currently running through direct written requests instead of listed
        availability.
      </p>
    </div>
  {/if}
</div>

<PhaseWorkToolbar>
  {#if showRequests}
    <RoundPlusButton
      label={reviewer ? 'Requests' : 'Your requests'}
      ariaLabel="Open service requests"
      onclick={() => {
        state.listOpen = true;
        void state.load();
      }}
    />
  {/if}
  {#if reviewer}
    <RoundPlusButton
      label="Request settings"
      ariaLabel="Request service settings changes"
      onclick={() => state.openSettings(settings)}
    />
  {/if}
  {#if canRequest}
    <RoundPlusButton
      standout
      label="Request service"
      ariaLabel="Request this service"
      onclick={() => state.openComposer(settings.mode)}
    />
  {/if}
</PhaseWorkToolbar>

<RequestsSheet {state} {reviewer} {requestsOff} {highlighted} />
<RequestComposer
  {state}
  scheduleRequired={settings.mode === 'calendar'}
  onSend={() => void send()}
/>
{#if reviewer}
  <RequestSettingsSheet {state} current={settings} onSave={() => void saveSettings()} />
{/if}
