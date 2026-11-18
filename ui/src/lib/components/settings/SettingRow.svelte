<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  /** One setting as a row (Settings as drawn, 2026-11-13): its name at the left, its value
   * and a chevron at the right; tapping opens the small sheet that changes it. */
  interface Props {
    label: string;
    value: string;
    /** The value is a key: shown whole, in small type. */
    key?: boolean;
    onOpen: () => void;
  }

  let { label, value, key = false, onOpen }: Props = $props();
</script>

<button
  class="set-row"
  type="button"
  aria-label={value && !key ? `${label}: ${value}` : label}
  onclick={onOpen}
>
  <span class="set-row-label">{t(label)}</span>
  <span class="set-row-value" class:key>{value}</span>
  <svg aria-hidden="true" viewBox="0 0 24 24" width="16" height="16">
    <path
      d="M9.5 6.5 15 12l-5.5 5.5"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
</button>
