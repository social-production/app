<script lang="ts">
  import ReportComposer from '#lib/components/moderation/ReportComposer.svelte';
  import type { ChatLine } from '#lib/domain/chat.ts';
  import type { ReportReason } from '#lib/domain/moderation.ts';

  import type { ChatReports } from './chatReports.ts';

  /** web LiveChatPanel's ReportComposerModal for the message picked in the bubble menu. */
  interface Props {
    target: ChatLine | null;
    reports: ChatReports;
    onClose: () => void;
  }

  let { target, reports, onClose }: Props = $props();

  let reason = $state<ReportReason>('spam');
  let description = $state('');
  let pending = $state(false);
  let error = $state('');

  function close(): void {
    reason = 'spam';
    description = '';
    error = '';
    onClose();
  }

  async function submit(): Promise<void> {
    if (!target) return;
    pending = true;
    error = '';
    try {
      await reports.report(target.id, reason, description);
      close();
    } catch (e) {
      error = reports.explain(e);
    } finally {
      pending = false;
    }
  }
</script>

<ReportComposer
  open={target !== null}
  itemLabel="comment"
  {pending}
  {error}
  bind:reason
  bind:description
  onClose={close}
  onSubmit={() => void submit()}
/>
