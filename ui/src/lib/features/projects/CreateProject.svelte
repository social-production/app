<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import CreatePreview from '#lib/components/create/CreatePreview.svelte';
  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import LocationIntent from '#lib/components/create/LocationIntent.svelte';
  import { searchFailure, searchPlaces } from '#lib/services/locations.ts';
  import PolicyNotice from '#lib/components/create/PolicyNotice.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import { tagCounts } from '#lib/domain/scopes.ts';
  import ScopeTagsStep from './scopes/ScopeTagsStep.svelte';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import { CREATE_STEPS, CreateState } from './createState.svelte.ts';

  /** web's CreateProjectPage without its Type step (ADR 0059), in the Creating Through Plans
   * mockup's order: basics → tags → location → overview. */
  const s = new CreateState();
  // The Platform page's "Start a platform project" (Platform and Moderators mockups §1).
  if (page.url.searchParams.get('tag') === 'platform') s.tags = ['#platform'];
  const steps = CREATE_STEPS;

  $effect(() => {
    void s.checkAllowance();
    // Convert (ADR 0106): `?continues=<id>` starts from the converted project.
    const continues = page.url.searchParams.get('continues');
    if (continues) void s.continueFrom(continues);
  });

  async function publish(): Promise<void> {
    const id = await s.submit();
    if (id) await goto(resolve('/projects/[id]', { id }));
  }
</script>

<CreateSheet
  title={t('New proposal')}
  description={t(
    'A proposal is shared work people can find, join and shape. Its plan decides what it becomes.',
  )}
  {steps}
  stepIndex={s.step}
  missing={s.missing}
  submitLabel={t('Publish proposal')}
  submitting={s.submitting}
  onStep={(i) => (s.step = i)}
  onSubmit={() => void publish()}
  onClose={() => void goto(resolve('/'))}
>
  {#if s.stepId === 'basics'}
    <div class="form-stack">
      <label>
        <RequiredLabel>{t('Title')}</RequiredLabel>
        <input
          bind:value={s.title}
          maxlength="200"
          aria-required="true"
          placeholder={t('Shared seed library')}
        />
      </label>
      <label>
        <RequiredLabel>{t('Description')}</RequiredLabel>
        <!-- The step asks "What do you propose?"; the field keeps that in its name. -->
        <textarea
          bind:value={s.description}
          rows="5"
          aria-required="true"
          aria-label={t('What do you propose? Description')}
          placeholder={t('The idea, and why it is worth doing.')}></textarea>
      </label>
      <p class="soft-note">
        <strong>{t('No type yet')}</strong>{t(
          'It becomes a project, a service or an event when its members approve a plan.',
        )}
      </p>
      <!-- The weekly limit shows only when it is close (Polish Pass mockups). -->
      {#if s.allowance !== null && s.allowance <= 1}
        <p class="helper-text">
          New members can start 2 proposals a week ({s.allowance} left). Full members have no limit.
        </p>
      {/if}
      <PolicyNotice />
    </div>
  {:else if s.stepId === 'location'}
    <LocationIntent
      intent={s.intent}
      place={s.placeValue}
      search={searchPlaces}
      failure={searchFailure}
      onIntent={(i) => (s.intent = i)}
      onPlace={(p) => s.setPlace(p)}
    />
  {:else if s.stepId === 'scope'}
    <div class="form-stack">
      <ScopeTagsStep tags={s.tags} onChange={(t) => (s.tags = t)} />
    </div>
  {:else}
    <div class="form-stack overview">
      <CreatePreview
        mode="proposal"
        scope={s.tags[0]?.slice(1) ?? ''}
        title={s.title.trim()}
        body={s.description.trim()}
        place={s.intent === 'physical' ? s.locationLabel : ''}
      />
      <button class="overview-row" type="button" onclick={() => (s.step = s.stepOf('scope'))}>
        <strong>{t('Channels')}</strong><span>{tagCounts(s.tags)}</span><em>{t('Edit')}</em>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = s.stepOf('location'))}>
        <strong>{t('Place')}</strong><span>{t(s.locationLabel)}</span><em>{t('Edit')}</em>
      </button>
      <PolicyNotice />
      {#if s.message}<p class="status-note">{s.message}</p>{/if}
    </div>
  {/if}
</CreateSheet>
