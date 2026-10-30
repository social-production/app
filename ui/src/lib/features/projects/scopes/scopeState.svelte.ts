import type { Contact } from '#lib/domain/messages.ts';
import { displayName, type Person } from '#lib/domain/people.ts';
import type { Project } from '#lib/domain/projects.ts';
import { parseInvite, scopeTag, type Scope, type ScopeKind } from '#lib/domain/scopes.ts';
import { closedCommunity, inviteToCommunity } from '#lib/services/closedCommunities.ts';
import { explain } from '#lib/services/errors.ts';
import { getPerson, myAgent } from '#lib/services/people.ts';
import { getProject, joinProject, joinScope } from '#lib/services/projects.ts';
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
  /** web's CommunityInvitePanel: the pasted link or code, and what happened. */
  inviteDraft = $state('');
  invitePending = $state(false);
  inviteFeedback = $state('');
  inviteWarning = $state(false);
  me = $state('');
  readonly feed: FeedState;

  /** `invite`: a closed community's seed from an invite link's fragment (ADR 0068). */
  constructor(
    readonly kind: ScopeKind,
    readonly slug: string,
    readonly invite: string | null = null,
  ) {
    this.feed = new FeedState(() => '', undefined, scopeTag(kind, slug));
    if (invite) this.inviteDraft = invite;
  }

  async open(): Promise<void> {
    try {
      const invited = this.invite && this.kind === 'community';
      this.scope = invited
        ? await closedCommunity(this.invite as string)
        : await findScope(this.slug, this.kind);
      this.loaded = true;
      if (!this.scope) return;
      this.gov = new ScopeGovernance(this.scope.id, () => !!this.project?.amMember);
      // Nothing can be tagged with a closed community yet, and the index can't read it.
      await Promise.all([this.load(), this.scope.isPrivate ? null : this.feed.reload()]);
      await this.gov.load();
    } catch (e) {
      this.error = this.invite
        ? `That invite link or code is invalid, or the community hasn't reached you yet. ${explain(e)}`
        : explain(e);
      this.loaded = true;
    }
  }

  /** After joining, removing someone or adding an organizer. */
  async reload(): Promise<void> {
    try {
      await this.load();
    } catch (e) {
      this.error = explain(e);
    }
  }

  private async load(): Promise<void> {
    if (!this.scope) return;
    this.project = await getProject(this.scope.id);
    this.me ||= await myAgent().catch(() => '');
    this.members = await Promise.all(this.project.members.map(getPerson));
  }

  /** Members other than you, by name (whom a private community's settings act on). */
  get others(): Contact[] {
    return this.members
      .filter((m) => m.agent !== this.me)
      .map((m) => ({ agent: m.agent, name: displayName(m) }));
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
      if (this.project.amMember) await joinProject(this.scope.id, false);
      else await joinScope(this.scope.id);
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

/** web's handleInviteRedeem: join the closed community a pasted link or code names. Returns
 * it, so the page can open it when it's another community. */
export async function redeemInvite(s: ScopeState): Promise<Scope | null> {
  const seed = parseInvite(s.inviteDraft);
  if (!seed || s.invitePending) {
    s.inviteFeedback = 'That invite link or code is invalid or expired.';
    s.inviteWarning = true;
    return null;
  }
  s.invitePending = true;
  s.inviteFeedback = '';
  try {
    const scope = await closedCommunity(seed);
    if (!scope) throw new Error('That invite link or code is invalid or expired.');
    await joinScope(seed);
    showMyScopes(await myScopes().catch(() => []));
    if (seed === s.scope?.id) await s.reload();
    return scope;
  } catch (e) {
    s.inviteFeedback = explain(e);
    s.inviteWarning = true;
    return null;
  } finally {
    s.invitePending = false;
  }
}

/** web's inviteUserToCommunity: an inbox invite; they join once they accept. */
export async function sendInvite(s: ScopeState, contact: Contact): Promise<void> {
  if (!s.scope || s.invitePending) return;
  s.invitePending = true;
  s.inviteFeedback = '';
  s.inviteWarning = false;
  try {
    if (s.project?.members.includes(contact.agent)) {
      s.inviteFeedback = `${contact.name} is already in this community.`;
      return;
    }
    await inviteToCommunity(contact.agent, s.scope.id);
    s.inviteFeedback = `Invite sent to ${contact.name}. They join once they accept.`;
  } catch {
    s.inviteFeedback = 'That user could not be invited. Check the username and try again.';
    s.inviteWarning = true;
  } finally {
    s.invitePending = false;
  }
}
