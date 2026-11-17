/** Directions to a place in the map apps people use (owner, 2026-11-18: "Google Maps, Apple
 * Maps or whatever the user has on the phone"). Pure: the sheet picks which to offer. */
export interface DirectionsApp {
  id: 'google' | 'apple' | 'waze' | 'phone' | 'osm';
  name: string;
  href: string;
}

export interface DirectionsTarget {
  latitude: number;
  longitude: number;
  label: string;
}

export type Platform = 'ios' | 'android' | 'other';

export function platformOf(userAgent: string): Platform {
  if (/iPhone|iPad|iPod|Macintosh/.test(userAgent)) return 'ios';
  if (/Android/.test(userAgent)) return 'android';
  return 'other';
}

/** The apps to offer, the phone's own first: Apple Maps on Apple devices, the default map
 * app (a `geo:` link, which Android hands to whatever app the person chose) on Android. */
export function directionsApps(to: DirectionsTarget, platform: Platform): DirectionsApp[] {
  const at = `${to.latitude},${to.longitude}`;
  const google: DirectionsApp = {
    id: 'google',
    name: 'Google Maps',
    href: `https://www.google.com/maps/dir/?api=1&destination=${at}`,
  };
  const apple: DirectionsApp = {
    id: 'apple',
    name: 'Apple Maps',
    href: `https://maps.apple.com/?daddr=${at}&q=${encodeURIComponent(to.label)}`,
  };
  const waze: DirectionsApp = {
    id: 'waze',
    name: 'Waze',
    href: `https://waze.com/ul?ll=${at}&navigate=yes`,
  };
  const phone: DirectionsApp = {
    id: 'phone',
    name: 'Your map app',
    href: `geo:${at}?q=${at}(${encodeURIComponent(to.label)})`,
  };
  const osm: DirectionsApp = {
    id: 'osm',
    name: 'OpenStreetMap',
    href: `https://www.openstreetmap.org/directions?to=${to.latitude}%2C${to.longitude}`,
  };
  if (platform === 'ios') return [apple, google, waze, osm];
  if (platform === 'android') return [phone, google, waze, osm];
  return [google, apple, waze, osm];
}
