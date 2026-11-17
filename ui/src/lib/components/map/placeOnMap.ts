import { getContext, setContext } from 'svelte';

/**
 * web's `openPlaceOnMap` for cards and sheets: "Show on map" opens the app's map panel on a
 * place (ADR 0085). Components stay presentational (CODE_RULES): the shell provides it.
 */
export interface PlaceOnMap {
  /** `select` is the page of the thing at the place: its pin opens selected. */
  show: (label: string, select?: string) => void;
}

const KEY = Symbol('place-on-map');

export function providePlaceOnMap(map: PlaceOnMap): void {
  setContext(KEY, map);
}

/** None outside the shell: the link then opens OpenStreetMap, as its href says. */
export function placeOnMap(): PlaceOnMap | undefined {
  return getContext<PlaceOnMap | undefined>(KEY);
}
