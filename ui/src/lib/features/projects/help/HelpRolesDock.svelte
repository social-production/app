<script lang="ts">
  import VoteStrip from '#lib/components/cards/VoteStrip.svelte';
  import DetailActionDock from '#lib/components/DetailActionDock.svelte';
  import type { ActivityRole } from '#lib/domain/activities.ts';
  import { slotsOf } from '#lib/domain/helpRequests.ts';

  import type { HelpState } from './helpState.svelte.ts';

  /**
   * web's HelpRequestRolesSection: each role with its sign-ups and a Sign up / Leave / Full
   * button, then the vote strip. web's Share menu comes with link sharing.
   */
  let { help }: { help: HelpState } = $props();

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
