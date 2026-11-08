import type {
  SoftwareGovernance,
  SoftwarePullRequest,
  SoftwareRequest,
} from '#lib/domain/software.ts';
import { explain } from '#lib/services/errors.ts';
import { castVote } from '#lib/services/governance.ts';
import { namesOf } from '#lib/services/people.ts';
import { loadSoftware, proposeSoftware } from '#lib/services/software.ts';

/** The five things web's SoftwareGovernanceWizard does. */
export type SoftwareMode =
  'create-pr' | 'vote-pr' | 'record-merge' | 'merge-capability' | 'repository-replacement';

/**
 * web's ProjectSoftwareGovernancePanel state (ADR 0086): the governance zome's `software`
 * read, the open wizard and its pull request, and the writes behind it. A pull request's
 * vote is the pull request's own while it is approved, then its recorded merge's.
 */
export class SoftwareState {
  governance = $state<SoftwareGovernance | null>(null);
  names = $state<Record<string, string>>({});
  error = $state('');
  busy = $state(false);
  mode = $state<SoftwareMode | null>(null);
  activeId = $state<string | null>(null);

  constructor(readonly projectId: string) {}

  readonly active = $derived(
    this.governance?.pullRequests.find((p) => p.id === this.activeId) ?? null,
  );

  async load(): Promise<void> {
    try {
      const g = await loadSoftware(this.projectId);
      const people = [
        ...g.mergers.map((m) => m.agent),
        ...g.candidates,
        ...g.pullRequests.flatMap((p) => [p.author, ...(p.merge ? [p.merge.by] : [])]),
        ...g.repositoryRequests.map((r) => r.author),
      ];
      const missing = people.filter((a, i) => people.indexOf(a) === i && !this.names[a]);
      if (missing.length > 0) this.names = { ...this.names, ...(await namesOf(missing)) };
      this.governance = g;
      this.error = '';
    } catch (e) {
      this.error = explain(e);
    }
  }

  open(mode: SoftwareMode, pr: SoftwarePullRequest | null = null): void {
    this.error = '';
    this.mode = mode;
    this.activeId = pr?.id ?? null;
  }

  close(): void {
    this.mode = null;
    this.activeId = null;
  }

  private async run(task: () => Promise<unknown>): Promise<boolean> {
    if (this.busy) return false;
    this.busy = true;
    try {
      await task();
      await this.load();
      return true;
    } catch (e) {
      this.error = explain(e);
      return false;
    } finally {
      this.busy = false;
    }
  }

  submit(request: SoftwareRequest): Promise<boolean> {
    return this.run(() => proposeSoftware(this.projectId, request));
  }

  /** The vote open on `pr`: its own, or its recorded merge's (the confirmation). */
  vote(pr: SoftwarePullRequest, choice: 'Yes' | 'No'): Promise<boolean> {
    const proposal = pr.stage === 'confirmation' && pr.merge ? pr.merge.proposal : pr.id;
    return this.run(() => castVote(this.projectId, proposal, choice));
  }
}
