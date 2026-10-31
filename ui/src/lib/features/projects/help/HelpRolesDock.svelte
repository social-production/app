<script lang="ts">
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

  /**
   * web's HelpRequestRolesSection: each role with its sign-ups and a Sign up / Leave / Full
   * button, then web's Share menu (signed in) and the vote strip.
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
      await shareHelpRequest(contact, help.project?.charter?.title ?? 'Help request', url);
      return null;
    } catch (e) {
      return explain(e);
    }
  }

  function label(role: ActivityRole): string {
    if (help.pending === role.index) return 'Working...';
    if (help.activity?.myRole === role.index) return 'Leave';
    return help.hasRoom(role) ? 'Sign up' : 'Full';
  }
</script>

<DetailActionDock active>
  <div class="help-roles">
    <div class="context-dock">
      {#if help.activity && help.activity.roles.length > 0}
        <div class="role-stack">
          {#each help.activity.roles as role (role.index)}
            {@const mine = help.activity.myRole === role.index}
            <article class="role-bar">
              <div class="role-copy">
                <strong>{role.label}</strong>
                {#if role.description}<p>{role.description}</p>{/if}
                <span>
                  {role.people.length} signed up
                  {#if slotsOf(role.maximum) > 0}· {slotsOf(role.maximum)} needed{/if}
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
      {/if}
      {#if help.project}
        <div class="signal-row">
          {#if signedIn}
            <ShareUserMenu
              copyLinkUrl={url}
              menuTitle="Share help request"
              compact
              shareTitle={help.project.charter?.title ?? null}
              searchContacts={search}
              submitShare={send}
            />
          {/if}
          <VoteStrip
            docked
            support={help.project.demand}
            oppose={help.project.opposition}
            mine={help.project.mySignal}
            onSignal={(next) => help.vote(next)}
          />
        </div>
      {/if}
    </div>
  </div>
</DetailActionDock>
{#if help.roleMessage}<div class="help-roles">
    <p class="role-message">{help.roleMessage}</p>
  </div>{/if}
