import type { FollowRow, FollowView } from '#lib/domain/follows.ts';
import type { Person } from '#lib/domain/people.ts';
import { explain } from '#lib/services/errors.ts';
import {
  decideFollow,
  follow,
  followRequests,
  followView,
  unfollow,
} from '#lib/services/follows.ts';
import { getPerson } from '#lib/services/people.ts';

export type PeopleList = 'followers' | 'following';

/** A profile's follows: the viewer's button, the counts, the people sheets and, on your own
 *  profile, the requests waiting for you (web's ProfilePage). */
export class ProfileFollows {
  view = $state<FollowView>();
  requests = $state<FollowRow[]>([]);
  people = $state<Record<string, Person>>({});
  list = $state<PeopleList | null>(null);
  busy = $state(false);
  deciding = $state<string | null>(null);
  error = $state<string>();

  constructor(
    private readonly agent: string,
    private readonly own: boolean,
  ) {}

  async load(): Promise<void> {
    try {
      const [view, requests] = await Promise.all([
        followView(this.agent),
        this.own ? followRequests() : Promise.resolve([]),
      ]);
      this.view = view;
      this.requests = requests;
      await this.learn([...view.followers, ...view.following, ...requests].map((r) => r.agent));
    } catch (e) {
      this.error = explain(e);
    }
  }

  /** Follow, or withdraw a follow or request (as web's single button does). */
  async toggle(): Promise<void> {
    await this.act(async () => {
      if (this.view?.myStatus) await unfollow(this.agent);
      else await follow(this.agent);
    });
  }

  async decide(follower: string, accept: boolean): Promise<void> {
    this.deciding = follower;
    try {
      await decideFollow(follower, accept);
      this.requests = this.requests.filter((r) => r.agent !== follower);
      await this.load();
    } catch {
      this.error = 'Could not update follow request. Reload and try again.';
    } finally {
      this.deciding = null;
    }
  }

  private async act(action: () => Promise<void>): Promise<void> {
    this.busy = true;
    this.error = undefined;
    try {
      await action();
      await this.load();
    } catch {
      this.error = 'Could not update follow status. Reload and try again.';
    } finally {
      this.busy = false;
    }
  }

  private async learn(agents: string[]): Promise<void> {
    const missing = agents.filter((a, i) => agents.indexOf(a) === i && !this.people[a]);
    const found = await Promise.all(missing.map((a) => getPerson(a).catch(() => undefined)));
    const next = { ...this.people };
    for (const person of found) if (person) next[person.agent] = person;
    this.people = next;
  }
}
