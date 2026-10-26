import {
  mentionTarget,
  rankMentions,
  type Mentionable,
  type MentionPool,
} from '#lib/domain/mentions.ts';

/** The chat being written in: you, its people, and the names it already knows. */
export interface MentionHere {
  me: string;
  agents: string[];
  names: Record<string, string>;
}

/**
 * Mention suggestions and links for one chat (ADR 0047). The pool is read once, when someone
 * types "@" or a message mentions someone, and again after a search finds nobody: a new
 * account can take a while to reach this node's directory.
 */
export class MentionSource {
  private pool = $state<MentionPool>({
    people: [],
    following: [],
    followers: [],
    partners: [],
  });
  private loading: Promise<MentionPool> | undefined;

  constructor(
    private readonly load: () => Promise<MentionPool>,
    private readonly here: () => MentionHere,
  ) {}

  ensure(): Promise<MentionPool> {
    this.loading ??= this.load().then(
      (pool) => (this.pool = pool),
      (e: unknown) => {
        this.loading = undefined;
        throw e;
      },
    );
    return this.loading;
  }

  /** Read the pool if a message mentions someone, so its link can be found. */
  watch(bodies: string[]): void {
    if (bodies.some((b) => b.includes('@'))) this.ensure().catch(() => undefined);
  }

  search = async (query: string): Promise<Mentionable[]> => {
    await this.ensure();
    const found = rankMentions(query, this.everyone(), this.circles(), this.here().me);
    if (found.length === 0) this.loading = undefined;
    return found;
  };

  agentOf = (name: string): string | null => mentionTarget(name, this.everyone(), this.circles());

  private everyone(): Mentionable[] {
    const known = Object.entries(this.here().names).map(([agent, name]) => ({ agent, name }));
    return [...known, ...this.pool.people];
  }

  private circles(): string[][] {
    const { following, followers, partners } = this.pool;
    return [this.here().agents, following, followers, partners];
  }
}
