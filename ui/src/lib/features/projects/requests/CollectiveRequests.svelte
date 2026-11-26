<script lang="ts">
  import { serviceMenu } from './serviceMenu.svelte.ts';
  import { t } from '#lib/i18n/index.ts';
  import PhaseMenu, { type PhaseMenuItem } from '#lib/components/governance/PhaseMenu.svelte';
  import PhaseWorkToolbar from '#lib/components/governance/PhaseWorkToolbar.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';
  import type { RequestSettings } from '#lib/domain/projects.ts';
  import type { Activity } from '#lib/domain/activities.ts';
  import { plannersSeed, type ServiceRequest } from '#lib/domain/serviceRequests.ts';
  import {
    inPlannersChat,
    openPlannersChat,
    syncPlannersChat,
  } from '#lib/services/requestChats.ts';

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
   * activity (the page's activity wizard), and Settings ▾ → Request settings, a change the
   * members vote on. A request stays open until planned, or reads Not met once its time passes
   * (ADR 0113). Its record lives in a holder chat with the members, hidden from Messages; the
   * talk happens in the planners' chat, which grows with the activity's roles.
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
  const showList = $derived(
    member ? settings.enabled || requests.shown.length > 0 : requests.shown.length > 0,
  );
  const requestItems = $derived<PhaseMenuItem[]>([
    {
      id: 'new-request',
      glyph: '+',
      label: t('New request'),
      detail: t('Ask the members for something; whoever plans it starts a chat with you.'),
      onSelect: openRequest,
    },
    {
      id: 'requests',
      glyph: '≡',
      label: member ? t('Requests') : t('Your requests ({n})', { n: requests.shown.length }),
      detail: member
        ? t('Everyone’s requests to this service, to plan or decline.')
        : t('What you asked for, and where each one stands.'),
      onSelect: showRequests,
    },
  ]);

  function showRequests(): void {
    requests.listOpen = true;
    void requests.load();
  }

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

  /** The page's wizard wrote the planned activity: the request is accepted in its holder
   * chat, and the planner opens the planners' chat with the person asking (ADR 0113). */
  function accept(requestId: string): void {
    const request = requests.requests.find((r) => r.id === requestId);
    if (!request) return;
    void requests
      .decide(request, 'accepted')
      .then(() => openPlannersChat(request))
      .catch(() => {});
  }

  /** The activity that planned each request. */
  const plannedBy = $derived(
    Object.fromEntries(
      activities.activities.flatMap((a) => (a.booking ? [[a.booking.request, a]] : [])),
    ) as Record<string, Activity>,
  );

  // The planner's device keeps each planners' chat in step with its activity's roles.
  let synced = '';
  $effect(() => {
    if (!member || !requests.loaded || !activities.loaded) return;
    const mine = requests.requests.filter((r) => plannedBy[r.id]?.author === me);
    const key = mine
      .map((r) => `${r.id}:${JSON.stringify(plannedBy[r.id]?.roles.map((x) => x.people))}`)
      .join('|');
    if (key === synced) return;
    synced = key;
    for (const r of mine) {
      const a = plannedBy[r.id];
      if (a) void syncPlannersChat(r, a).catch(() => {});
    }
  });

  /** A request's conversation: the planners' chat, once planned and if you are in it. */
  function chatOf(r: ServiceRequest): string | null {
    const a = plannedBy[r.id];
    return a && inPlannersChat(me, r, a) ? plannersSeed(a.author, r.id) : null;
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
      title: project.charter?.title ?? t('Service'),
      creator: project.creator,
      settings,
      members: project.members,
    });
    if (sent) requests.listOpen = true;
  }

  async function voteSettings(): Promise<void> {
    if (await requests.saveSettings()) await detail.load();
  }
</script>

<!-- One Request segment, never green (Join is the dock's call to action), owner-approved
     mockup 2026-11-26: with a list to show as well it is "Request ▾"/"Requests ▾". -->
<PhaseWorkToolbar>
  {#snippet governance()}
    {#if canRequest && showList}
      <PhaseMenu
        items={requestItems}
        label={member ? t('Requests') : t('Request')}
        sheetTitle={member ? t('Requests') : t('Request')}
        labelledById="request-menu-title"
      />
    {:else if canRequest}
      <RoundPlusButton
        plus={false}
        label={t('Request')}
        ariaLabel={t('Request service')}
        onclick={openRequest}
      />
    {:else if showList}
      <RoundPlusButton
        plus={false}
        label={member ? t('Requests') : t('Your requests')}
        ariaLabel={t('Show service requests')}
        onclick={showRequests}
      />
    {/if}
  {/snippet}
</PhaseWorkToolbar>

<RequestsSheet
  state={requests}
  reviewer={member}
  requestsOff={!settings.enabled}
  {highlighted}
  onPlan={member && canPlan ? plan : undefined}
  {chatOf}
/>
<RequestComposer
  state={requests}
  scheduleRequired={settings.mode === 'calendar'}
  onSend={() => void send()}
/>
{#if member}
  <OverlaySheet
    open={serviceMenu.open}
    title={t('Settings')}
    labelledById="service-settings-menu"
    rootClass="collective-service"
    onClose={() => (serviceMenu.open = false)}
  >
    <div class="settings-menu">
      <button
        class="settings-option"
        type="button"
        onclick={() => {
          serviceMenu.open = false;
          requests.openSettings(settings);
        }}
      >
        <strong>{t('Request settings')}</strong>
        <span>{t('Turn requests on or off, and change how people ask for this service.')}</span>
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
