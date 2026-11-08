/** How web's map pins look (`maps/maplibre.ts`): colours by kind and mode, the tooltip. */
import { formatMarkerScheduleRange } from '#lib/domain/markerTime.ts';
import type { MapMarker } from './types.ts';

const MARKER_COLORS: Record<string, string> = {
  project: '#16a34a',
  help_request: '#e07a5f',
  activity: '#2563eb',
};
const EVENT_COLOR = '#5b21b6';
const EVENT_ACTIVITY_COLOR = '#a78bfa';

const PROJECT_MODE_PROJECT_COLORS: Record<string, string> = {
  'collective-service': '#a16207',
  'personal-service': '#7c2d12',
};
const PRODUCTIVE_PROJECT_COLOR = '#166534';

const PROJECT_MODE_ACTIVITY_COLORS: Record<string, string> = {
  'collective-service': '#eab308',
  'personal-service': '#ea580c',
};
const PRODUCTIVE_ACTIVITY_COLOR = '#22c55e';

export function resolveMarkerColor(item: MapMarker): string {
  const mode = item.projectMode ?? 'productive';
  if (item.entityType === 'project') {
    return PROJECT_MODE_PROJECT_COLORS[mode] ?? PRODUCTIVE_PROJECT_COLOR;
  }
  if (item.entityType === 'activity' && item.activitySource === 'project') {
    return PROJECT_MODE_ACTIVITY_COLORS[mode] ?? PRODUCTIVE_ACTIVITY_COLOR;
  }
  if (item.entityType === 'activity' && item.activitySource === 'event') {
    return EVENT_ACTIVITY_COLOR;
  }
  return MARKER_COLORS[item.entityType ?? 'event'] ?? EVENT_COLOR;
}

function escapeHtml(value: string) {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
}

export function truncate(value: string, maxLength: number) {
  const trimmed = value.trim();
  if (trimmed.length <= maxLength) {
    return trimmed;
  }
  return `${trimmed.slice(0, maxLength - 1)}…`;
}

function signupLine(item: MapMarker) {
  if (item.entityType === 'help_request' && item.slotsNeeded != null) {
    return `${item.signupCount ?? 0}/${item.slotsNeeded} signed up`;
  }
  if (item.entityType === 'activity' && item.minimumParticipants != null) {
    return `${item.committedCount ?? 0}/${item.minimumParticipants} committed`;
  }
  return '';
}

export function buildTooltipHtml(item: MapMarker) {
  const lines: string[] = [];
  if (item.parentTitle?.trim()) {
    lines.push(`<strong>${escapeHtml(truncate(item.parentTitle, 48))}</strong>`);
    lines.push(`<div>${escapeHtml(truncate(item.label, 56))}</div>`);
  } else {
    lines.push(`<strong>${escapeHtml(truncate(item.label, 56))}</strong>`);
  }
  if (item.statusText?.trim()) {
    lines.push(
      `<div class="sp-map-tooltip__meta">${escapeHtml(truncate(item.statusText, 56))}</div>`,
    );
  }
  const schedule = formatMarkerScheduleRange(item.scheduledAt, item.endsAt);
  if (schedule) {
    lines.push(`<div class="sp-map-tooltip__meta">${escapeHtml(schedule)}</div>`);
  }
  if (item.displayLabel?.trim()) {
    lines.push(
      `<div class="sp-map-tooltip__meta">${escapeHtml(truncate(item.displayLabel, 56))}</div>`,
    );
  }
  const signup = signupLine(item);
  if (signup) {
    lines.push(`<div class="sp-map-tooltip__meta">${escapeHtml(signup)}</div>`);
  }
  const children = item.children ?? [];
  if (children.length > 0) {
    for (const child of children.slice(0, 4)) {
      lines.push(
        `<div class="sp-map-tooltip__meta">${escapeHtml(truncate(child.label, 48))}</div>`,
      );
    }
    if (children.length > 4) {
      lines.push(`<div class="sp-map-tooltip__meta">+${children.length - 4} more</div>`);
    }
  }
  return lines.join('');
}

export function ensureTooltipStyles() {
  if (typeof document === 'undefined' || document.getElementById('sp-map-tooltip-styles')) {
    return;
  }

  const style = document.createElement('style');
  style.id = 'sp-map-tooltip-styles';
  style.textContent = `
    .sp-map-tooltip {
      position: fixed;
      z-index: 200;
      pointer-events: none;
      max-width: 240px;
      padding: 8px 10px;
      border: 1px solid rgba(15, 23, 42, 0.12);
      border-radius: 10px;
      background: rgba(255, 255, 255, 0.98);
      color: #111827;
      box-shadow: 0 10px 24px rgba(15, 23, 42, 0.16);
      font: 12px/1.35 system-ui, sans-serif;
    }
    .sp-map-tooltip strong {
      display: block;
      margin-bottom: 2px;
      font-size: 12px;
    }
    .sp-map-tooltip__meta {
      margin-top: 4px;
      color: #4b5563;
      font-size: 11px;
    }
  `;
  document.head.appendChild(style);
}
