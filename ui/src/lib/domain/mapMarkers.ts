/** web's `MapMarkerItem` (`services/adapters/types.ts` `getMapMarkers`), from the gateway's
 * `/index/map-markers` (ADR 0082, 0083): snake case and microseconds in, web's camel case and
 * ISO times out, so the ported panel reads them as web's does. */

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
