<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  /**
   * How moderators work (Platform and Moderators mockups §3): five numbered steps, the last
   * saying plainly what a moderator does (web's explainer, ADR 0095's rules).
   */
  let { open, onClose }: { open: boolean; onClose: () => void } = $props();

  const steps = [
    ['Volunteer', 'Any Full member can put themselves forward.'],
    [
      'Members vote',
      'Every Full member can vote yes or no on each person, and change it any time.',
    ],
    [
      'A seat',
      'Enough votes (sized from the people active last week) and at least 66% yes. With nobody active last week, a volunteer is seated straight away.',
    ],
    [
      "A week's grace",
      'If votes fall short but stay mostly yes, a seat lasts a week before it is lost.',
    ],
    [
      'What moderators do',
      "Merge the changes members approved on platform software projects. Nothing else: they don't decide on their own, and they can't block anything.",
    ],
  ];
</script>

<OverlaySheet
  {open}
  title={t('How moderators work')}
  labelledById="board-how-title"
  rootClass="platform-board"
  elevated
  {onClose}
>
  <ol class="board-steps">
    {#each steps as [title, body], i (title)}
      <li>
        <span class="board-step-num" aria-hidden="true">{i + 1}</span>
        <div>
          <b>{title}</b>
          <p>{body}</p>
        </div>
      </li>
    {/each}
  </ol>
</OverlaySheet>
