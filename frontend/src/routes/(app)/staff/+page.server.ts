import type { PageServerLoad } from './$types';
import { api } from '$lib/server/api';
import type { RoomOverview } from '$lib/api/types';

export const load: PageServerLoad = async ({ locals, fetch }) => {
  const rooms = await api<RoomOverview[]>('/rooms/overview', { token: locals.token, fetch });

  // Lowest floor first, rooms without a floor last, then room number in natural order
  // so "2" comes before "10" and "A-3" before "A-12".
  rooms.sort((a, b) => {
    const fa = a.room.floor ?? Number.POSITIVE_INFINITY;
    const fb = b.room.floor ?? Number.POSITIVE_INFINITY;
    if (fa !== fb) return fa - fb;
    return a.room.room_number.localeCompare(b.room.room_number, undefined, { numeric: true });
  });

  return { rooms, loadedAt: new Date().toISOString() };
};
