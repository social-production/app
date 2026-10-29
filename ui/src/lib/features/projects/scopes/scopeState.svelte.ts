import type { Person } from '#lib/domain/people.ts';
import type { Project } from '#lib/domain/projects.ts';
import { scopeTag, type Scope, type ScopeKind } from '#lib/domain/scopes.ts';
import { explain } from '#lib/services/errors.ts';
import { getPerson } from '#lib/services/people.ts';
import { getProject, joinProject } from '#lib/services/projects.ts';
import { findScope } from '#lib/services/scopes.ts';
import { requireViewer } from '#lib/stores/authNotice.svelte.ts';

import { FeedState } from '../feedState.svelte.ts';

/** One channel's or community's page (web's ScopePageData): the scope, its members, its feed. */
export class ScopeState {
  scope = $state<Scope | null>(null);
  project = $state<Project>();
  members = $state<Person[]>([]);
  error = $state<string>();
  loaded = $state(false);
  joining = $state(false);
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
      await Promise.all([this.load(), this.feed.reload()]);
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

  /** web's handleMembershipToggle: join, or leave. */
  async toggleJoin(): Promise<void> {
    if (!this.scope || !this.project || this.joining) return;
    if (!requireViewer(`Sign in to join ${this.scope.handle}.`)) return;
    this.joining = true;
    try {
      await joinProject(this.scope.id, !this.project.amMember);
      await this.load();
    } catch (e) {
      this.error = explain(e);
    } finally {
      this.joining = false;
    }
  }
}
