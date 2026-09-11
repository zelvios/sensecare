import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { api, ApiError } from '$lib/server/api';
import {
  type Measurement,
  PERIOD_HOURS,
  type PeriodHours,
  type RoomOverview,
  type ServiceCall,
  type Threshold
} from '$lib/api/types';

async function orNull<T>(p: Promise<T>): Promise<T | null> {
  try {
    return await p;
  } catch (e) {
    if (e instanceof ApiError && e.status === 404) return null;
    throw e;
  }
}

export const load: PageServerLoad = async ({ locals, params, url, fetch }) => {
  const token = locals.token;
  const requested = Number(url.searchParams.get('hours'));
  const hours: PeriodHours = PERIOD_HOURS.includes(requested as PeriodHours)
    ? (requested as PeriodHours)
    : 24;

  const to = new Date();
  const from = new Date(to.getTime() - hours * 3600 * 1000);

  const [overview, latest, thresholds, history, calls] = await Promise.all([
    api<RoomOverview[]>('/rooms/overview?include_inactive=true', { token, fetch }),
    orNull(api<Measurement>(`/rooms/${params.id}/measurements/latest`, { token, fetch })),
    orNull(api<Threshold>(`/rooms/${params.id}/thresholds`, { token, fetch })),
    api<Measurement[]>(
      `/rooms/${params.id}/measurements?from=${from.toISOString()}&to=${to.toISOString()}&limit=500`,
      { token, fetch }
    ),
    api<ServiceCall[]>(`/service-calls?room_id=${params.id}&limit=50`, { token, fetch })
  ]);

  const room = overview.find((r) => r.room.id === params.id);
  if (!room) error(404, 'Room not found');

  return {
    room,
    latest,
    thresholds,
    history,
    hours,
    calls,
    canManage: locals.user?.role === 'admin'
  };
};
