<script lang="ts">
  import type { Snippet } from 'svelte';

  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import PhotoViewer from '#lib/components/overlay/PhotoViewer.svelte';
  import { displayName, type Person } from '#lib/domain/people.ts';

  import { standingHint, standingLabel } from './standing.ts';

  /** The header of web's ProfilePage (classes styled by styles/profile.css). */
  interface Props {
    person: Person;
    devices?: number;
    actions?: Snippet;
    /** Leading items of the count row (web's follower / following buttons). */
    counts?: Snippet;
    /** web's standing chip is a button that opens the Trust sheet. */
    trustOpen?: boolean;
    onStanding?: () => void;
    /** The cover photo's URL (ADR 0107). */
    cover?: string | null;
    /** Your own profile: the cover and the photo get camera buttons. */
    onCoverFile?: ((file: File) => void) | undefined;
    onPhotoFile?: ((file: File) => void) | undefined;
  }

  let {
    person,
    devices = 1,
    actions,
    counts,
    trustOpen = false,
    onStanding,
    cover = null,
    onCoverFile,
    onPhotoFile,
  }: Props = $props();

  const picked = (event: Event, then?: (file: File) => void): void => {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (file) then?.(file);
    input.value = '';
  };

  // web's "View <name> profile photo": the picture opens full screen in the PhotoViewer.
  const avatar = $derived(person.profile?.avatar ?? null);
  let photoOpen = $state(false);
  const photoUrl = $derived(
    photoOpen && avatar
      ? URL.createObjectURL(new Blob([avatar.bytes.slice()], { type: avatar.mime }))
      : null,
  );
  $effect(() => () => {
    if (photoUrl) URL.revokeObjectURL(photoUrl);
  });

  import '#lib/components/styles/profile.css';
</script>

{#snippet camera(label: string, onFile: (file: File) => void, cls: string)}
  <label class="hero-cam {cls}" aria-label={label} title={label}>
    <svg aria-hidden="true" viewBox="0 0 24 24"
      ><rect x="3" y="5" width="18" height="14" rx="2" /><circle cx="9" cy="10" r="2" /><path
        d="m21 16-5-5-8 8"
      /></svg
    >
    <input hidden type="file" accept="image/*" onchange={(e) => picked(e, onFile)} />
  </label>
{/snippet}

<section class="hero-section" class:has-cover={!!cover}>
  <div class="profile-cover" style:background-image={cover ? `url(${cover})` : null}>
    {#if onCoverFile}{@render camera('Change cover photo', onCoverFile, 'cover-cam')}{/if}
  </div>
  <div class="hero-topline">
    <div class="hero-main">
      <div class="hero-identity">
        {#if avatar}
          <button
            class="avatar-open"
            type="button"
            aria-label={`View ${person.profile?.name || displayName(person)} profile photo`}
            onclick={() => (photoOpen = true)}
          >
            <AvatarBadge size="md" name={person.profile?.name ?? ''} {avatar} />
          </button>
        {:else}
          <AvatarBadge size="md" name={person.profile?.name ?? ''} avatar={null} />
        {/if}
        {#if onPhotoFile}{@render camera('Change profile photo', onPhotoFile, 'photo-cam')}{/if}
        <div class="hero-copy">
          <div class="name-row">
            <h1>{displayName(person)}</h1>
            {#if onStanding}
              <button
                aria-expanded={trustOpen}
                aria-label={`Standing: ${standingLabel(person)}`}
                class="standing-chip"
                class:paused={person.standing.level === 'Paused'}
                title={standingHint(person)}
                type="button"
                onclick={onStanding}>{standingLabel(person)}</button
              >
            {:else}
              <span
                class="standing-chip"
                class:paused={person.standing.level === 'Paused'}
                title={standingHint(person)}>{standingLabel(person)}</span
              >
            {/if}
          </div>
          {#if person.profile?.bio}
            <p class="profile-bio">{person.profile.bio}</p>
          {/if}
          <div class="count-row">
            {@render counts?.()}
            {#if person.profile?.home_area}<span>{person.profile.home_area}</span>{/if}
            {#if devices > 1}<span><strong>{devices}</strong> devices</span>{/if}
            {#if person.standing.level === 'New'}
              <span><strong>{person.standing.vouchers.length}</strong>/2 vouches</span>
            {/if}
          </div>
        </div>
      </div>
      {@render actions?.()}
    </div>
  </div>
</section>

<PhotoViewer
  url={photoUrl}
  alt={`${person.profile?.name || displayName(person)} profile photo`}
  onClose={() => (photoOpen = false)}
/>
