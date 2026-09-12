import * as m from '$lib/paraglide/messages';

/** The fields the search looks at. Both `Room` and `RoomOverview['room']` satisfy this. */
export type RoomLike = { room_number: string; name?: string | null; floor?: number | null };

export type FloorFilter = 'all' | number;

/** Distinct floors present, lowest first. Rooms without a floor are not listed. */
export function floorsOf(rooms: RoomLike[]): number[] {
  return [...new Set(rooms.map((r) => r.floor).filter((f): f is number => f != null))].sort(
    (a, b) => a - b
  );
}

/**
 * Filters rooms by the floor select and the free-text query.
 * "etage 2" / "floor 2" (the floor word in the active language) narrows to that floor only.
 * A bare number matches a floor exactly or a room number and name anywhere.
 */
export function filterRooms<T extends RoomLike>(
  rooms: T[],
  query: string,
  floor: FloorFilter
): T[] {
  const onFloor = floor === 'all' ? rooms : rooms.filter((r) => r.floor === floor);

  const q = query.trim().toLowerCase();
  if (!q) return onFloor;

  const floorWord = m.floor().toLowerCase();
  const byFloor = q.match(new RegExp(`^${floorWord}\\s*(\\d+)$`));
  if (byFloor) {
    const wanted = Number(byFloor[1]);
    return onFloor.filter((r) => r.floor === wanted);
  }

  return onFloor.filter((r) => {
    const f = r.floor == null ? '' : String(r.floor);
    return (
      f === q || r.room_number.toLowerCase().includes(q) || (r.name ?? '').toLowerCase().includes(q)
    );
  });
}
