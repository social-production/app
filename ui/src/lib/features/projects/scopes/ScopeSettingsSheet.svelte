<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { APPROVAL_PERCENT } from '#lib/domain/governance.ts';
  import type { Contact } from '#lib/domain/messages.ts';
  import type { ScopeKind } from '#lib/domain/scopes.ts';

  import type { ScopeArt } from './scopeArt.svelte.ts';
  import ScopeDecisionCard from './ScopeDecisionCard.svelte';
  import ScopeSheetArt from './ScopeSheetArt.svelte';
  import type { ScopeGovernance } from './scopeGovernance.svelte.ts';
  import ScopeWhoDecides from './ScopeWhoDecides.svelte';

  import '#lib/components/styles/scopes.css';

  /**
   * A scope's Settings (the pencil), redrawn like Edit profile (Channel Pictures mockups §2):
   * the cover band and mark on top, Name (fixed: tags use it, ADR 0068) and Description, one
   * line saying who decides, and the action in the sheet's dock. Decisions lists open and past
   * votes as cards. A private community adds web's "Who decides"; an organizer-run one's
   * organizers edit directly. The picture and cover go the same way (ADR 0109).
   */
  interface Props {
    open: boolean;
    kind: ScopeKind;
    handle: string;
    description: string;
    isMember: boolean;
    gov: ScopeGovernance;
    memberName: (agent: string) => string;
    /** Its picture and cover, and the new ones staged here (ADR 0109). */
    art: ScopeArt;
    /** A private community's governance (ADR 0068 part d). */
    isPrivate?: boolean;
    organized?: boolean;
    amOrganizer?: boolean;
    organizerNames?: string[];
    members?: Contact[];
    /** After a change that applied at once (an organizer's edit, removal or new organizer). */
    onChanged?: () => void;
    onClose: () => void;
  }

  let {
    open,
    kind,
    handle,
    description,
    isMember,
    gov,
    memberName,
    art,
    isPrivate = false,
    organized = false,
    amOrganizer = false,
    organizerNames = [],
    members = [],
    onChanged = () => {},
    onClose,
  }: Props = $props();

  /** web's viewerEditsDirectly; others in an organizer-run community see a read-only form. */
  const direct = $derived(organized && amOrganizer);
  const canEdit = $derived(isMember && (!organized || direct));

  let tab = $state<'settings' | 'decisions'>('settings');
  let draft = $state('');
  let notice = $state('');

  $effect(() => {
    if (!open) return;
    draft = description;
    notice = '';
    how = false;
    art.clearStaged();
    void gov.load();
  });

  const described = $derived(draft.trim() !== description.trim());
  const changed = $derived(described || !!art.newPicture || !!art.newCover);

  let how = $state(false);
  const rule = $derived(
    direct
      ? 'Organizers decide: saved at once'
      : organized
        ? t('Organizers decide: read only for you')
        : `Members vote: ${APPROVAL_PERCENT}% yes`,
  );

  async function save(): Promise<void> {
    const change = {
      description: described ? draft.trim() : null,
      picture: art.newPicture?.hash ?? null,
      cover: art.newCover?.hash ?? null,
    };
    if (await gov.proposeEdit(change)) {
      art.clearStaged();
      notice = direct ? t('Saved') : t('Vote started');
      if (direct) onChanged();
      else tab = 'decisions';
    }
  }
</script>

<OverlaySheet
  {open}
  title={t('Settings')}
  labelledById="scope-settings-title"
  rootClass="scope-settings"
  {onClose}
>
  {#snippet toolbar()}
    <div class="sheet-tabs" role="tablist" aria-label={t('Settings sections')}>
      <button
        type="button"
        role="tab"
        aria-selected={tab === 'settings'}
        class:active={tab === 'settings'}
        onclick={() => (tab = 'settings')}>{t('Settings')}</button
      >
      <button
        type="button"
        role="tab"
        aria-selected={tab === 'decisions'}
        class:active={tab === 'decisions'}
        onclick={() => (tab = 'decisions')}
      >
        {t('Decisions')}
        {#if gov.open.length}<span class="tab-count">{gov.open.length}</span>{/if}
      </button>
    </div>
  {/snippet}
  <div class="settings-body">
    {#if gov.message}<p class="error">{gov.message}</p>{/if}
    {#if notice}<p class="how">{notice}</p>{/if}
    {#if tab === 'decisions'}
      <section class="decisions" aria-label={t('Decisions')}>
        {#if !gov.loaded}
          <SkeletonRows count={2} kind="row" />
        {:else if gov.open.length === 0 && gov.past.length === 0}
          <p class="empty-card">{t('No decisions yet.')}</p>
        {/if}
        {#if gov.open.length > 0}
          <h3 class="decisions-head">{t('Open')}</h3>
          {#each gov.open as item (item.id)}
            <ScopeDecisionCard
              {item}
              {memberName}
              {art}
              busy={gov.busy}
              onVote={isMember && item.myVote === null
                ? (choice) => void gov.vote(item.id, choice)
                : undefined}
            />
          {/each}
          {#if !isMember}<p class="vote-note">
              {t('Members vote on these. Join to take part.')}
            </p>{/if}
        {/if}
        {#if gov.past.length > 0}
          <h3 class="decisions-head">{t('Past')}</h3>
          {#each gov.past as item (item.id)}<ScopeDecisionCard {item} {memberName} {art} />{/each}
        {/if}
      </section>
    {:else}
      <ScopeSheetArt {kind} {art} {canEdit} />

      <label class="field">
        <span>{t('Name')}</span>
        <input value={handle} maxlength="32" aria-describedby="scope-name-hint" disabled />
        <span id="scope-name-hint" class="hint"
          >{t('Tags use this name, so it stays as it is.')}</span
        >
      </label>
      <label class="field">
        <span>{t('Description')}</span>
        <textarea bind:value={draft} rows="3" maxlength="2000" disabled={!canEdit}></textarea>
      </label>

      <div class="scope-rule-strip">
        <ToolbarIcon name="people" />
        <span>{rule}</span>
        <button type="button" aria-expanded={how} onclick={() => (how = !how)}>{t('How')}</button>
      </div>
      {#if how}
        <p class="how">
          {#if direct}
            {t("You're an organizer, so changes apply straight away.")}
          {:else if organized}
            Organizers ({organizerNames.join(', ')}) look after these settings.
          {:else}
            Collaborative: changes go to a member vote ({APPROVAL_PERCENT}% yes, with quorum based
            on members).
          {/if}
        </p>
      {/if}

      {#if isPrivate}
        <ScopeWhoDecides
          {organized}
          {direct}
          {isMember}
          {organizerNames}
          {members}
          {gov}
          onChanged={(n) => {
            notice = n;
            if (direct) onChanged();
            else tab = 'decisions';
          }}
        />
      {/if}
    {/if}
  </div>
  {#snippet footer()}
    {#if canEdit && tab === 'settings'}
      <div class="sheet-actions">
        <button class="sheet-submit" type="button" disabled={gov.busy || !changed} onclick={save}>
          {direct ? t('Save') : t('Propose change')}
        </button>
      </div>
    {/if}
  {/snippet}
</OverlaySheet>
