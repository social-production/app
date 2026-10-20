/**
 * Project values (ADR 0041): web's Proposal-phase "Add value" and 1–10 importance votes.
 * Mirrors the governance zome's `ValueView`; wording is web's (ProjectValueCard.svelte).
 */

export type Importance = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10;

export interface ProjectValue {
  id: string;
  author: string;
  createdAt: number;
  label: string;
  /** Importance votes from members with Full standing. */
  voteCount: number;
  /** web's `importanceScore`: the average, 0 when unrated. */
  importanceScore: number;
  /** web's `activeImportanceVote`: the viewer's vote, 0 when none. */
  activeImportanceVote: Importance | 0;
}

/** web's AddValueSheet input limit (sp_governance::values::LABEL_MAX). */
export const VALUE_LABEL_MAX = 160;

/** web's `importanceOptions` (ProjectLifecyclePanel.svelte). */
export const IMPORTANCE_OPTIONS: { value: Importance; label: string }[] = Array.from(
  { length: 10 },
  (_, index) => {
    const value = (index + 1) as Importance;
    return {
      value,
      label: value === 1 ? 'Unnecessary' : value === 10 ? 'Required' : `Importance ${value} of 10`,
    };
  },
);

/** web's `sortValuesForRating` (utils/sortDetailValues.ts). */
export function sortValuesForRating(values: ProjectValue[]): ProjectValue[] {
  return [...values].sort((left, right) => {
    const leftNeedsVote = left.activeImportanceVote <= 0;
    const rightNeedsVote = right.activeImportanceVote <= 0;
    if (leftNeedsVote !== rightNeedsVote) return leftNeedsVote ? 1 : -1;
    if (right.importanceScore !== left.importanceScore) {
      return right.importanceScore - left.importanceScore;
    }
    return left.label.localeCompare(right.label);
  });
}
