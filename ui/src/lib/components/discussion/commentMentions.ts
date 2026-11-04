import type { Mentionable } from '#lib/domain/mentions.ts';

/** web's comment mentions: suggestions while typing, and "@name" linking to the profile. */
export interface CommentMentions {
  search: (query: string) => Promise<Mentionable[]>;
  agentOf: (name: string) => string | null;
}
