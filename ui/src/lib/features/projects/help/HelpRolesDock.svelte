<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import VoteStrip from '#lib/components/cards/VoteStrip.svelte';
  import DetailActionDock from '#lib/components/DetailActionDock.svelte';
  import ShareUserMenu from '#lib/components/overlay/ShareUserMenu.svelte';
  import type { ActivityRole } from '#lib/domain/activities.ts';
  import { slotsOf } from '#lib/domain/helpRequests.ts';
  import type { Contact } from '#lib/domain/messages.ts';
  import { explain } from '#lib/services/errors.ts';
  import { shareHelpRequest } from '#lib/services/helpRequests.ts';
  import { contacts } from '#lib/services/messages.ts';
  import { myAgent } from '#lib/services/people.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import type { HelpState } from './helpState.svelte.ts';
  import GuestShare from '../GuestShare.svelte';

  /**
   * web's HelpRequestRolesSection as drawn (Invites, Help Requests and Software Screens, 2):
   * how many roles are filled, each role as a soft card with who is in and Join / Leave / Full,
   * and the dock with the vote strip and web's Share menu. Detail Pages Cleanup §4: the dock
   * has no "Join a role" (each role has its own Join) and the heading is sentence case.
   */
  let { help }: { help: HelpState } = $props();

  const signedIn = $derived(connection.current.status === 'connected');
  const url = $derived(`${location.origin}${location.pathname}`);
  let people: Promise<Contact[]> | null = null;

  async function search(query: string): Promise<Contact[]> {
    people ??= Promise.all([contacts(), myAgent()]).then(([all, me]) =>
      all.filter((c) => c.agent !== me),
    );
    const needle = query.trim().toLowerCase();
    return (await people).filter((c) => c.name.toLowerCase().includes(needle)).slice(0, 8);
  }

  async function send(contact: Contact): Promise<string | null> {
    try {
      await shareHelpRequest(contact, help.project?.charter?.title ?? t('Help request'), url);
      return null;
    } catch (e) {
      return explain(e);
    }
  }

  const roles = $derived(help.activity?.roles ?? []);
  const filled = $derived(roles.filter((r) => !help.hasRoom(r)).length);

  function label(role: ActivityRole): string {
    if (help.pending === role.index) return t('Working...');
    if (help.activity?.myRole === role.index) return t('Leave');
    return help.hasRoom(role) ? t('Join') : t('Full');
  }
</script>

<!-- ADR 0099: the roles are part of the page; the dock holds the votes and Share. -->
{#if help.activity && help.activity.roles.length > 0}
  <div class="help-roles help-roles-page">
    <div class="detail-section-head">
      <h2>{t('Roles')}</h2>
      <span>{filled} of {roles.length} filled</span>
    </div>
    <div class="role-stack">
      {#each help.activity.roles as role (role.index)}
        {@const mine = help.activity.myRole === role.index}
        <article class="role-bar">
          <div class="role-copy">
            <strong>{role.label}</strong>
            {#if role.description}<p>{role.description}</p>{/if}
            <span>
              {slotsOf(role.maximum) > 0 ? `${slotsOf(role.maximum)} needed · ` : ''}{role.people
                .length
                ? role.people.map((a) => help.names[a] ?? '…').join(', ')
                : t('nobody yet')}
            </span>
          </div>
          <button
            class:selected={mine}
            class="signup"
            disabled={help.pending === role.index ||
              !help.activity.rolesOpen ||
              (!mine && !help.hasRoom(role))}
            type="button"
            onclick={() => void help.toggle(role)}
          >
            {label(role)}
          </button>
        </article>
      {/each}
    </div>
  </div>
{/if}
<DetailActionDock active>
  {#if help.project}
    <VoteStrip
      bare
      support={help.project.demand}
      oppose={help.project.opposition}
      mine={help.project.mySignal}
      onSignal={(next) => help.vote(next)}
    />
    {#if signedIn}
      <ShareUserMenu
        copyLinkUrl={url}
        menuTitle={t('Share help request')}
        compact
        shareTitle={help.project.charter?.title ?? null}
        searchContacts={search}
        submitShare={send}
      />
    {:else}
      <GuestShare title={help.project.charter?.title ?? ''} />
    {/if}
  {/if}
</DetailActionDock>
{#if help.roleMessage}<div class="help-roles">
    <p class="role-message">{help.roleMessage}</p>
  </div>{/if}
