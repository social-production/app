/** web's `MapMarkerItem` (`services/adapters/types.ts` `getMapMarkers`), from the gateway's
 * `/index/map-markers` (ADR 0082, 0083): snake case and microseconds in, web's camel case and
 * ISO times out, so the ported panel reads them as web's does. */
import type { MapMarker } from '#lib/maps/types.ts';

export type MarkerEntity = 'event' | 'project' | 'help_request' | 'activity';

export interface MapMarkerItem {
  id: string;
  entityType: MarkerEntity;
  activitySource: 'event' | 'project' | null;
  projectMode: string | null;
  title: string;
  parentId: string | null;
  parentTitle: string | null;
  subtitle: string | null;
  href: string;
  latitude: number;
  longitude: number;
  precision: string;
  displayLabel: string;
  distanceKm: number;
  scheduledAt: string | null;
  endsAt: string | null;
  signupCount: number | null;
  slotsNeeded: number | null;
  committedCount: number | null;
  minimumParticipants: number | null;
}

/** What the gateway sends (`routes/map.rs` `Marker`). */
export interface GatewayMarker {
  id: string;
  entity_type: string;
  activity_source?: string | null;
  project_mode?: string | null;
  title: string;
  parent_id?: string | null;
  parent_title?: string | null;
  subtitle?: string | null;
  href: string;
  latitude: number;
  longitude: number;
  precision: string;
  display_label: string;
  distance_km: number;
  scheduled_at?: number | null;
  ends_at?: number | null;
  signup_count?: number | null;
  slots_needed?: number | null;
  committed_count?: number | null;
  minimum_participants?: number | null;
}

/** Holochain microseconds to an ISO time. */
export const isoOfMicros = (micros: number | null | undefined): string | null =>
  typeof micros === 'number' ? new Date(Math.floor(micros / 1000)).toISOString() : null;

const ENTITIES: readonly string[] = ['project', 'help_request', 'activity'];

export function markerItem(m: GatewayMarker): MapMarkerItem {
  return {
    id: m.id,
    entityType: (ENTITIES.includes(m.entity_type) ? m.entity_type : 'event') as MarkerEntity,
    activitySource:
      m.activity_source === 'project' || m.activity_source === 'event' ? m.activity_source : null,
    projectMode: m.project_mode ?? null,
    title: m.title,
    parentId: m.parent_id ?? null,
    parentTitle: m.parent_title ?? null,
    subtitle: m.subtitle ?? null,
    href: m.href,
    latitude: m.latitude,
    longitude: m.longitude,
    precision: m.precision,
    displayLabel: m.display_label,
    distanceKm: m.distance_km,
    scheduledAt: isoOfMicros(m.scheduled_at),
    endsAt: isoOfMicros(m.ends_at),
    signupCount: m.signup_count ?? null,
    slotsNeeded: m.slots_needed ?? null,
    committedCount: m.committed_count ?? null,
    minimumParticipants: m.minimum_participants ?? null,
  };
}

const PROJECT_MODES = ['productive', 'collective-service', 'personal-service'] as const;
type ProjectModeName = (typeof PROJECT_MODES)[number];

/** web's `MapDiscoveryCard` `mapEntityKind`, as the app's card mode (label and accent). */
export function markerMode(
  item: Pick<MapMarkerItem, 'entityType' | 'activitySource' | 'projectMode'>,
): ProjectModeName | 'event' | 'help-request' {
  const project = (PROJECT_MODES as readonly string[]).includes(item.projectMode ?? '')
    ? (item.projectMode as ProjectModeName)
    : 'productive';
  if (item.entityType === 'project') return project;
  if (item.entityType === 'activity' && item.activitySource === 'project') return project;
  if (item.entityType === 'help_request') return 'help-request';
  return 'event';
}

/** web's card footer: "1.2 km", then sign-ups or "N going · needs M more". */
export function markerFooter(item: MapMarkerItem): string {
  let signups = '';
  if (item.entityType === 'help_request' && item.slotsNeeded != null) {
    signups = `${item.signupCount ?? 0}/${item.slotsNeeded} signed up`;
  } else if (item.entityType === 'activity' && item.minimumParticipants != null) {
    const going = item.committedCount ?? 0;
    const min = item.minimumParticipants;
    signups = going < min ? `${going} going · needs ${min - going} more` : `${going} going`;
  }
  return [`${item.distanceKm.toFixed(1)} km`, signups].filter(Boolean).join(' · ');
}

/** web's status line: a project pin's subtitle ("Proposal", "Open spots"). */
export const markerStatus = (item: MapMarkerItem): string =>
  item.entityType === 'project' && item.activitySource == null ? (item.subtitle ?? '').trim() : '';

/** The app route a marker's `href` names (`/projects/<id>?activity=<id>`, …). */
export function markerRoute(href: string): {
  path: 'projects' | 'events' | 'help-requests';
  id: string;
  query: string;
} | null {
  const m = /^\/(projects|events|help-requests)\/([^/?#]+)(\?[^#]*)?$/.exec(href);
  if (!m?.[1] || !m[2]) return null;
  return { path: m[1] as 'projects' | 'events' | 'help-requests', id: m[2], query: m[3] ?? '' };
}

/** web's `mapMarkerFromItem`: what a pin shows. */
export function pinOf(item: MapMarkerItem): MapMarker {
  const activity = item.entityType === 'activity';
  return {
    id: item.id,
    latitude: item.latitude,
    longitude: item.longitude,
    label: item.title,
    href: item.href,
    entityType: item.entityType,
    activitySource: item.activitySource,
    projectMode: item.projectMode,
    parentId: activity ? item.parentId : null,
    parentTitle: activity ? item.parentTitle : null,
    statusText: item.entityType === 'project' && !item.activitySource ? item.subtitle : null,
    displayLabel: item.displayLabel,
    scheduledAt: item.scheduledAt,
    endsAt: item.endsAt,
    signupCount: item.signupCount,
    slotsNeeded: item.slotsNeeded,
    committedCount: item.committedCount,
    minimumParticipants: item.minimumParticipants,
  };
}

/** web's `sortListItemsBySchedule`: soonest first, untimed last. */
export const bySchedule = (items: MapMarkerItem[]): MapMarkerItem[] =>
  [...items].sort((a, b) => {
    const at = (i: MapMarkerItem): number =>
      i.scheduledAt ? new Date(i.scheduledAt).getTime() : Number.POSITIVE_INFINITY;
    return at(a) - at(b);
  });
