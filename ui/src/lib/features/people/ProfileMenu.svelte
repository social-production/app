<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import ChatMenu, { type ChatMenuItem } from '#lib/components/messages/ChatMenu.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { shareLink } from '#lib/components/overlay/shareLink.ts';
  import { standingHint, standingLabel } from '#lib/components/people/standing.ts';
  import type { Person } from '#lib/domain/people.ts';

  import { resolve } from '$app/paths';

  import '#lib/components/styles/ux/chatmenus.css';

  /**
   * The ⋮ on a profile (owner, 2026-11-24), opening the chat header's floating menu: Share (the
   * app's one share sheet), the person's standing (it opens the Trust sheet; the page shows no
   * tag), and on someone else's profile Vouch for them and Mark as bot, with the old buttons'
   * shield and robot. A row you can't use yet stays, greyed, with the reason under it.
   */
  interface Props {
    person: Person;
    /** Signed out: Share and standing only. */
    viewer: Person | undefined;
    own: boolean;
    /** You marked them. */
    marked: boolean;
    canMark: boolean;
    busy: boolean;
    onStanding: () => void;
    onVouch: () => void;
    onMark: () => void;
  }

  let p: Props = $props();

  let at = $state<{ x: number; y: number } | null>(null);
  const sheet = shareLink();
  const name = $derived(p.person.profile?.name || 'them');

  function share(): void {
    const path = resolve('/profile/[agent]', { agent: p.person.agent });
    if (sheet) sheet.open({ kind: 'profile', title: name, path });
    else void navigator.share?.({ url: new URL(path, location.origin).href }).catch(() => {});
  }

  function vouchRow(viewer: Person): ChatMenuItem {
    const vouched = p.person.standing.vouchers.includes(viewer.agent);
    const why = vouched
      ? ''
      : p.person.standing.level === 'Full'
        ? t('Already a full member.')
        : p.person.standing.level === 'Paused'
          ? t('A community review found this account is a bot.')
          : viewer.standing.level !== 'Full'
            ? t('Only full members can vouch.')
            : '';
    return {
      label: vouched ? t('You vouched for {name}', { name }) : t('Vouch for {name}', { name }),
      icon: 'vouch',
      disabled: p.busy || vouched || !!why,
      ...(why ? { hint: why } : {}),
      onSelect: p.onVouch,
    };
  }

  function markRow(): ChatMenuItem {
    const why = p.marked || p.canMark ? '' : t('Opens once two full members vouch for you.');
    return {
      label: p.marked ? t('Clear bot mark') : t('Mark {name} as a bot', { name }),
      icon: 'bot',
      danger: true,
      disabled: p.busy || !!why,
      ...(why ? { hint: why } : {}),
      onSelect: p.onMark,
    };
  }

  const items = $derived.by((): ChatMenuItem[] => {
    const list: ChatMenuItem[] = [
      { label: t('Share profile'), icon: 'share', onSelect: share },
      {
        label: standingLabel(p.person),
        icon: p.person.standing.level === 'Full' ? 'person-check' : 'user',
        hint: standingHint(p.person),
        onSelect: p.onStanding,
      },
    ];
    const viewer = p.viewer;
    if (!p.own && viewer && viewer.standing.level !== 'Paused')
      list.push(vouchRow(viewer), markRow());
    return list;
  });
</script>

<button
  aria-expanded={!!at}
  aria-haspopup="menu"
  aria-label={t('More for {name}', { name })}
  class="profile-more"
  type="button"
  onclick={(event) => {
    const box = event.currentTarget.getBoundingClientRect();
    at = at ? null : { x: box.right - 240, y: box.bottom + 6 };
  }}
>
  <ToolbarIcon name="kebab" />
</button>

{#if at}
  <ChatMenu x={at.x} y={at.y} {items} onClose={() => (at = null)} />
{/if}
