/**
 * A card's short place (owner, 2026-11-17: "full addresses in cards can be a bit
 * overwhelming"): the geocoder's "Preston Market, 30, Cramer Street, Preston, Melbourne, City
 * of Darebin, Victoria, 3072, Australia" becomes "Preston Market, Preston". The name (or
 * number and street) comes first, then the suburb; numbers, postcodes, councils, states and
 * the country go. A typed place with no commas is left as it is.
 */
const STREET =
  /\b(street|st|road|rd|avenue|ave|lane|ln|parade|pde|drive|dr|way|place|pl|crescent|cres|highway|hwy|boulevard|blvd|terrace|tce|court|ct|grove|gr|close|square|sq|alley|walk)\.?$/i;
const NUMBER = /^\d+[a-z]?(-\d+[a-z]?)?$/i;
const REGION = /^(city of |shire of |municipality of )/i;

export function shortPlace(label: string | null | undefined): string {
  const raw = (label ?? '')
    .split(',')
    .map((p) => p.trim())
    .filter(Boolean);
  if (raw.length <= 2) return raw.join(', ');
  const head = raw[0] ?? '';
  const next = raw[1] ?? '';
  // "14, Elm Street, …" reads as "14 Elm Street".
  const first = NUMBER.test(head) && STREET.test(next) ? `${head} ${next}` : head;
  const rest = raw.slice(first === head ? 1 : 2);
  const suburb = rest.find(
    (p) => !NUMBER.test(p) && !STREET.test(p) && !REGION.test(p) && p !== first,
  );
  return suburb ? `${first}, ${suburb}` : first;
}
