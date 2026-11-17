<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import GroupsIcon from '#lib/components/icons/GroupsIcon.svelte';
  import type { Contact } from '#lib/domain/messages.ts';
  import PlaceStanding from '#lib/components/people/PlaceStanding.svelte';
  import { clashingNames, pickerName } from '#lib/domain/personLine.ts';
  import { cachedPerson } from '#lib/services/people.ts';

  /** New group's last step, as Telegram's: name the group, see who is in it, create it. */
  interface Props {
    members: Contact[];
    onBack: () => void;
    onCreate: (members: Contact[], title: string) => Promise<void>;
  }

  let { members, onBack, onCreate }: Props = $props();

  let title = $state('');
  let creating = $state(false);
  let error = $state('');

  async function create(): Promise<void> {
    if (creating) return;
    if (!title.trim()) {
      error = 'Give the group a name.';
      return;
    }
    creating = true;
    error = '';
    try {
      await onCreate(members, title.trim());
    } catch (e) {
      error = e instanceof Error ? e.message : 'The group could not be created.';
    } finally {
      creating = false;
    }
  }
</script>

<div class="new-message">
  <header class="new-message-head">
    <button
      class="round-back"
      type="button"
      aria-label="Back"
      title="Back"
      disabled={creating}
      onclick={onBack}
    >
      <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M15 5l-7 7 7 7" /></svg>
    </button>
    <div class="new-message-title">
      <h1>New group</h1>
      <p>{members.length} {members.length === 1 ? 'member' : 'members'}</p>
    </div>
  </header>

  <form
    class="group-name"
    onsubmit={(event) => {
      event.preventDefault();
      void create();
    }}
  >
    <span class="group-glyph"><GroupsIcon /></span>
    <input
      bind:value={title}
      type="text"
      aria-label="Group name"
      placeholder="Group name"
      maxlength="200"
      {@attach (node) => node.focus()}
    />
  </form>
  {#if error}<p class="new-message-note error" role="alert">{error}</p>{/if}
  {#if creating}<p class="new-message-note">Creating the group…</p>{/if}

  <div class="new-message-list">
    {#each members as person (person.agent)}
      <div class="person-row">
        <span class="person-face"><AvatarBadge size="md" name={person.name} /></span>
        <span class="person-copy">
          <strong>{pickerName(person, clashingNames(members))}</strong>
          <small><PlaceStanding agent={person.agent} load={cachedPerson} /></small>
        </span>
      </div>
    {/each}
  </div>

  <button
    class="message-fab"
    type="button"
    aria-label="Create group"
    title="Create group"
    disabled={creating}
    onclick={() => void create()}
  >
    <svg aria-hidden="true" viewBox="0 0 24 24">
      <path
        d="m5 12.5 4.5 4.5L19 7.5"
        fill="none"
        stroke="currentColor"
        stroke-width="2.2"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>
</div>
