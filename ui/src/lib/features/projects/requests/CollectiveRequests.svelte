<script lang="ts">
  import PhaseWorkToolbar from '#lib/components/governance/PhaseWorkToolbar.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';
  import type { RequestSettings } from '#lib/domain/projects.ts';
  import type { ServiceRequest } from '#lib/domain/serviceRequests.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import type { ActivitiesState } from '../activitiesState.svelte.ts';
  import type { DetailState } from '../detailState.svelte.ts';
  import RequestComposer from './RequestComposer.svelte';
  import RequestSettingsSheet from './RequestSettingsSheet.svelte';
  import RequestsSheet from './RequestsSheet.svelte';
  import { ServiceRequestsState } from './serviceRequestsState.svelte.ts';

  import '#lib/components/styles/collective-service.css';
  import '#lib/components/styles/service-requests.css';

  /**
   * A collective service's requests (web's CollectiveServicePhaseFive, ADR 0089): Request
   * service for anyone signed in, the Requests sheet where any member plans a request into an
   * activity (the page's activity wizard) or declines it, and Settings ▾ → Request settings,
   * a change the members vote on. Each request lives in a group chat with the members.
   */
  interface Props {
    detail: DetailState;
    activities: ActivitiesState;
    me: string;
    /** Members may plan requests now (the Activity phase). */
    canPlan: boolean;
    /** Hands the page `accept`, called once its wizard wrote a planned activity. */
    register: (accept: (requestId: string) => void) => void;
  }

  let { detail, activities, me, canPlan, register }: Props = $props();

  const project = $derived(detail.project);
  const member = $derived(!!me && !!project?.amMember);
  // svelte-ignore state_referenced_locally
  const requests = new ServiceRequestsState(
    detail.id,
    () => me,
    () => member,
  );
  // web's collective default: requests off until the members vote them on.
  const fallback: RequestSettings = { enabled: false, mode: 'both', offSchedule: false };
  const settings = $derived(project?.requests ?? fallback);
  const highlighted = $derived(page.url.searchParams.get('request'));
  const others = $derived((project?.members ?? []).filter((m) => m !== me));
  const canRequest = $derived(settings.enabled && others.length > 0);
  let menuOpen = $state(false);

  $effect(() => {
    if (me) void requests.load();
  });
  $effect(() => {
    if (member && requests.loaded && activities.loaded) {
      void requests.healPlanned(activities.activities.flatMap((a) => a.booking?.request ?? []));
    }
  });
  $effect(() => {
    if (highlighted && me) requests.listOpen = true;
  });

  /** The page's wizard wrote the planned activity: the request is accepted in its chat. */
  function accept(requestId: string): void {
    const request = requests.requests.find((r) => r.id === requestId);
    if (request) void requests.decide(request, 'accepted');
  }
  $effect(() => register(accept));

  function plan(request: ServiceRequest): void {
    requests.listOpen = false;
    activities.openForRequest(request, project?.charter?.location_label ?? '');
  }

  function openRequest(): void {
    if (!me) {
      void goto(resolve('/onboarding'));
      return;
    }
    requests.openComposer(settings.mode);
  }

  async function send(): Promise<void> {
    if (!project) return;
    const sent = await requests.send({
      title: project.charter?.title ?? 'Service',
      creator: project.creator,
      settings,
      slots: [],
      members: project.members,
    });
    if (sent) requests.listOpen = true;
  }

  async function voteSettings(): Promise<void> {
    if (await requests.saveSettings()) await detail.load();
  }
</script>

<PhaseWorkToolbar>
  {#if canRequest}
    <RoundPlusButton
      standout
      label="Request service"
      ariaLabel="Request service"
      onclick={openRequest}
    />
  {/if}
  {#if member ? settings.enabled || requests.shown.length > 0 : requests.shown.length > 0}
    <RoundPlusButton
      label={member ? 'Requests' : 'Your requests'}
      ariaLabel="Show service requests"
      onclick={() => {
        requests.listOpen = true;
        void requests.load();
      }}
    />
  {/if}
  {#if member}
    <RoundPlusButton
      label="Settings ▾"
      ariaLabel="Service settings"
      onclick={() => (menuOpen = true)}
    />
  {/if}
</PhaseWorkToolbar>

<RequestsSheet
  state={requests}
  reviewer={member}
  requestsOff={!settings.enabled}
  {highlighted}
  onPlan={member && canPlan ? plan : undefined}
/>
<RequestComposer
  state={requests}
  scheduleRequired={settings.mode === 'calendar'}
  onSend={() => void send()}
/>
{#if member}
  <OverlaySheet
    open={menuOpen}
    title="Settings"
    labelledById="service-settings-menu"
    rootClass="collective-service"
    onClose={() => (menuOpen = false)}
  >
    <div class="settings-menu">
      <button
        class="settings-option"
        type="button"
        onclick={() => {
          menuOpen = false;
          requests.openSettings(settings);
        }}
      >
        <strong>Request settings</strong>
        <span>Turn requests on or off, and change how people ask for this service.</span>
      </button>
    </div>
  </OverlaySheet>
  <RequestSettingsSheet
    state={requests}
    current={settings}
    vote
    onSave={() => void voteSettings()}
  />
{/if}
