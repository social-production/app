import { explain } from '#lib/services/errors.ts';
import { addOrganizer, removeOrganizer } from '#lib/services/events.ts';

import type { DetailState } from '../detailState.svelte.ts';

/** Make organiser / Remove on a private event (ADR 0061, 0073), with grants written here kept
 * until the network read catches up (a write lags its own reads). */
export class OrganizerActions {
  promoted = $state<string[]>([]);
  removed = $state<string[]>([]);
  pending = $state<string | null>(null);
  error = $state('');

  constructor(private detail: DetailState) {}

  organizers(): string[] {
    const keys = [...(this.detail.project?.organizers ?? []), ...this.promoted];
    return keys.filter((k, i) => keys.indexOf(k) === i && !this.removed.includes(k));
  }

  async promote(agent: string): Promise<void> {
    await this.run(agent, async () => {
      await addOrganizer(this.detail.id, agent);
      this.promoted = [...this.promoted, agent];
      this.removed = this.removed.filter((k) => k !== agent);
    });
  }

  async demote(agent: string): Promise<void> {
    await this.run(agent, async () => {
      await removeOrganizer(this.detail.id, agent);
      this.removed = [...this.removed, agent];
      this.promoted = this.promoted.filter((k) => k !== agent);
    });
  }

  private async run(agent: string, write: () => Promise<void>): Promise<void> {
    this.pending = agent;
    this.error = '';
    try {
      await write();
      void this.detail.load();
    } catch (e) {
      this.error = explain(e);
    } finally {
      this.pending = null;
    }
  }
}
