/**
 * Reports (web's ⋮ Report, ADR 0071): the discussion zome's `ModerationView`, and web's
 * `utils/moderation.ts` labels and threshold lines.
 */

export type ReportReason = 'spam' | 'serious-harm';
export type ReportResolution = 'open' | 'under_review' | 'hidden' | 'removed' | 'dismissed';
export type ModerationState = 'visible' | 'under_review' | 'hidden' | 'removed';

/** web's `ContentReportSummary` (shares as fractions, as web's are). */
export interface ContentReport {
  id: string;
  reporter: string;
  reason: ReportReason;
  description: string;
  createdAt: number;
  resolution: ReportResolution;
  yesCount: number;
  noCount: number;
  activeVote: 'yes' | 'no' | null;
  audienceSize: number;
  deleteQuorum: number;
  hideQuorum: number;
  deleteYesShare: number;
  hideYesShare: number;
}

export interface Moderation {
  state: ModerationState;
  report: ContentReport | null;
}

export const VISIBLE: Moderation = { state: 'visible', report: null };

/** A card's moderation field from the index (absent while nothing is reported). */
export function cardModeration(state: string | undefined): ModerationState {
  return state === 'under_review' || state === 'hidden' || state === 'removed' ? state : 'visible';
}

/** web's `moderationStatusLabel`. */
export function moderationStatusLabel(
  state: ModerationState,
  report: ContentReport | null,
): 'Under review' | 'Hidden' | null {
  if (state === 'hidden' || report?.resolution === 'hidden') return 'Hidden';
  if (state === 'under_review' || (report && report.resolution !== 'dismissed')) {
    return 'Under review';
  }
  return null;
}

/** web's ReportMenu `resolutionLabel`. */
export function resolutionLabel(resolution: ReportResolution): string {
  switch (resolution) {
    case 'hidden':
      return 'Hidden';
    case 'removed':
      return 'Removed';
    case 'dismissed':
      return 'Dismissed';
    default:
      return 'Under review';
  }
}

export function reasonLabel(reason: ReportReason): string {
  return reason === 'serious-harm' ? 'Serious harm' : 'Spam';
}

/** web's `formatReportThresholdLines`. */
export function thresholdLines(report: ContentReport): string[] {
  const audience = report.audienceSize;
  const note = (quorum: number) =>
    audience > quorum ? ` · sized from audience of ~${audience}` : '';
  const lines = [
    `Delete at ${report.deleteQuorum} votes cast · ${Math.round(report.deleteYesShare * 100)}% yes${note(report.deleteQuorum)}`,
  ];
  if (report.reason === 'serious-harm' && report.resolution !== 'removed') {
    const prefix = report.resolution === 'hidden' ? 'Hidden at' : 'Hide at';
    lines.unshift(
      `${prefix} ${report.hideQuorum} votes cast · ${Math.round(report.hideYesShare * 100)}% yes${note(report.hideQuorum)}`,
    );
  }
  return lines;
}

/** web's vote count line under the thresholds. */
export function voteCountLine(report: ContentReport): string {
  const total = report.yesCount + report.noCount;
  return `Current: ${report.yesCount} yes · ${report.noCount} no · ${total} total`;
}

/** web's `removed_placeholder`: what stands in for an item a report took down. */
export function removedLabel(reason: ReportReason | undefined): string {
  return `Removed for ${reason === 'serious-harm' ? 'serious harm' : 'spam'}`;
}

/** web's report `itemLabel` for a space's mode. */
export function itemLabelOf(mode: string): string {
  switch (mode) {
    case 'event':
      return 'event';
    case 'help-request':
      return 'help request';
    case 'thread':
      return 'thread';
    case 'post':
      return 'post';
    default:
      return 'project';
  }
}
