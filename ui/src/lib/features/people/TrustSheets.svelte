<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import ConfirmDialog from '#lib/components/overlay/ConfirmDialog.svelte';
  import PeopleSheet from '#lib/components/overlay/PeopleSheet.svelte';
  import { standingHint, standingLabel } from '#lib/components/people/standing.ts';
  import type { Person } from '#lib/domain/people.ts';

  import type { ProfileTrust } from './trustState.svelte.ts';

  /** web's Trust sheet (Vouchers / Bot marks tabs) and the "Mark … as a possible bot?" confirm. */
  let { trust, person }: { trust: ProfileTrust; person: Person } = $props();

  const name = $derived(person.profile?.name ?? 'them');
</script>

<PeopleSheet
  description={`${standingLabel(person)}. ${standingHint(person)} Only full members can vouch or mark bots. Three bot marks open a public review that anyone can vote in; marks alone never pause anyone.`}
  open={trust.open}
  sectionLayout="tabs"
  sections={[
    { title: 'Vouchers', emptyCopy: 'No vouches yet.', people: trust.vouchers },
    { title: 'Bot marks', emptyCopy: 'No bot marks yet.', people: trust.markers },
  ]}
  title={t('Trust')}
  onClose={() => (trust.open = false)}
/>
<ConfirmDialog
  open={trust.confirming}
  title={t('Mark {name} as a possible bot?', { name })}
  body={t(
    'One mark only flags the account. When three full members mark it, a public review opens and anyone can vote on whether it is a bot. You can clear your mark any time.',
  )}
  confirmLabel={t('Mark as bot')}
  danger
  onSettle={(ok) => (ok ? void trust.setMark(true) : (trust.confirming = false))}
/>
