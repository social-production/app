<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import type { Contact } from '#lib/domain/messages.ts';

  /** One member on a group's page: face, name, Manager pill, and for a manager a ⋮ with a
   * floating menu (Make manager or Stop managing, View profile, Remove from group). */
  interface Props {
    member: Contact;
    you: boolean;
    manager: boolean;
    canManage: boolean;
    /** Keep the ⋮ column even without a menu, so every Manager pill lines up. */
    keepSlot?: boolean;
    menuOpen: boolean;
    profile: string;
    onMenu: (open: boolean) => void;
    onRemove: () => void;
    onManager?: () => void;
  }

  let p: Props = $props();
  let row = $state<HTMLElement | null>(null);

  // A tap anywhere outside the row closes its menu.
  $effect(() => {
    if (!p.menuOpen) return;
    const away = (e: PointerEvent) => {
      if (row && !row.contains(e.target as Node)) p.onMenu(false);
    };
    window.addEventListener('pointerdown', away);
    return () => window.removeEventListener('pointerdown', away);
  });
</script>

<div class="group-row group-member" bind:this={row}>
  <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by the page -->
  <a class="group-person" href={p.profile}>
    <AvatarBadge size="md" name={p.member.name} agent={p.member.agent} />
    <span class="group-person-copy">
      <b>{p.member.name}{p.you ? t(' (you)') : ''}</b>
    </span>
  </a>
  {#if p.manager}<span class="group-role">{t('Manager')}</span>{/if}
  {#if p.canManage}
    <button
      class="group-more"
      type="button"
      aria-label={t('More for {name}', { name: p.member.name })}
      aria-haspopup="menu"
      aria-expanded={p.menuOpen}
      onclick={() => p.onMenu(!p.menuOpen)}
    >
      <svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true"
        ><circle cx="12" cy="5" r="1.8" fill="currentColor" /><circle
          cx="12"
          cy="12"
          r="1.8"
          fill="currentColor"
        /><circle cx="12" cy="19" r="1.8" fill="currentColor" /></svg
      >
    </button>
    {#if p.menuOpen}
      <div class="group-menu" role="menu">
        {#if p.onManager}
          <button role="menuitem" type="button" onclick={p.onManager}
            >{p.manager ? t('Stop managing') : t('Make manager')}</button
          >
        {/if}
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by the page -->
        <a role="menuitem" href={p.profile}>{t('View profile')}</a>
        <button role="menuitem" class="danger" type="button" onclick={p.onRemove}
          >{t('Remove from group')}</button
        >
      </div>
    {/if}
  {:else if p.keepSlot}
    <span class="group-more group-more-slot" aria-hidden="true"></span>
  {/if}
</div>
