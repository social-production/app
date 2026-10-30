import { displayName, type Person } from '#lib/domain/people.ts';
import type { Project } from '#lib/domain/projects.ts';
import { scopeTag, type Scope, type ScopeKind } from '#lib/domain/scopes.ts';
import { explain } from '#lib/services/errors.ts';
import { getPerson } from '#lib/services/people.ts';
import { getProject, joinProject } from '#lib/services/projects.ts';
import { findScope, myScopes } from '#lib/services/scopes.ts';
import { requireViewer } from '#lib/stores/authNotice.svelte.ts';
import { showMyScopes } from '#lib/stores/myScopes.svelte.ts';

import { FeedState } from '../feedState.svelte.ts';
import { ScopeGovernance } from './scopeGovernance.svelte.ts';

/** One channel's or community's page (web's ScopePageData): the scope, its members, its feed. */
export class ScopeState {
  scope = $state<Scope | null>(null);
  project = $state<Project>();
  members = $state<Person[]>([]);
  error = $state<string>();
  loaded = $state(false);
  joining = $state(false);
  /** Its decisions (web's scope settings), once the scope is found. */
  gov = $state<ScopeGovernance>();
  readonly feed: FeedState;

  constructor(
    readonly kind: ScopeKind,
    readonly slug: string,
  ) {
    this.feed = new FeedState(() => '', undefined, scopeTag(kind, slug));
  }

  async open(): Promise<void> {
    try {
      this.scope = await findScope(this.slug, this.kind);
      this.loaded = true;
      if (!this.scope) return;
      this.gov = new ScopeGovernance(this.scope.id, () => !!this.project?.amMember);
      await Promise.all([this.load(), this.feed.reload()]);
      await this.gov.load();
    } catch (e) {
      this.error = explain(e);
      this.loaded = true;
    }
  }

  private async load(): Promise<void> {
    if (!this.scope) return;
    this.project = await getProject(this.scope.id);
    this.members = await Promise.all(this.project.members.map(getPerson));
  }

  /** A member's name for decisions that name them. */
  memberName = (agent: string): string => {
    const person = this.members.find((m) => m.agent === agent);
    return person ? displayName(person) : 'a member';
  };

  /** web's handleMembershipToggle: join, or leave. */
  async toggleJoin(): Promise<void> {
    if (!this.scope || !this.project || this.joining) return;
    if (!requireViewer(`Sign in to join ${this.scope.handle}.`)) return;
    this.joining = true;
    try {
      await joinProject(this.scope.id, !this.project.amMember);
      await this.load();
      // The left rail lists what you joined.
      showMyScopes(await myScopes().catch(() => []));
    } catch (e) {
      this.error = explain(e);
    } finally {
      this.joining = false;
    }
  }
}
