import { error } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { api, ApiError } from '$lib/server/api';
import { act } from '$lib/server/actions';
import { PERIOD_HOURS } from '$lib/api/types';
import type {
  Measurement,
  PeriodHours,
  RoomOverview,
  ServiceCall,
  Stay,
  Threshold,
  User
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

  const [overview, latest, thresholds, history, calls, stays, clients] = await Promise.all([
    api<RoomOverview[]>('/rooms/overview?include_inactive=true', { token, fetch }),
    orNull(api<Measurement>(`/rooms/${params.id}/measurements/latest`, { token, fetch })),
    orNull(api<Threshold>(`/rooms/${params.id}/thresholds`, { token, fetch })),
    api<Measurement[]>(
      `/rooms/${params.id}/measurements?from=${from.toISOString()}&to=${to.toISOString()}&limit=500`,
      { token, fetch }
    ),
    api<ServiceCall[]>(`/service-calls?room_id=${params.id}&limit=50`, { token, fetch }),
    api<Stay[]>('/stays?open=true&limit=200', { token, fetch }),
    api<User[]>('/users?role=client&active=true&limit=200', { token, fetch })
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
    stay: stays.find((s) => s.room_id === params.id) ?? null,
    clients,
    occupiedIn: Object.fromEntries(stays.map((s) => [s.user_id, s.room_number])),
    canManage: locals.user?.role === 'admin'
  };
};

export const actions: Actions = {
  acknowledgeCall: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() =>
      api(`/service-calls/${id}/acknowledge`, { method: 'POST', token: locals.token, fetch })
    );
  },
  closeCall: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    const id = String(form.get('id'));
    const note = String(form.get('note') ?? '').trim() || undefined;
    return act(() =>
      api(`/service-calls/${id}/close`, {
        method: 'POST',
        body: { note },
        token: locals.token,
        fetch
      })
    );
  },
  acknowledgeAlarm: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() =>
      api(`/alarms/${id}/acknowledge`, { method: 'POST', token: locals.token, fetch })
    );
  },
  resolveAlarm: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() => api(`/alarms/${id}/resolve`, { method: 'POST', token: locals.token, fetch }));
  },
  checkIn: async ({ request, locals, params, fetch }) => {
    const user_id = String((await request.formData()).get('user_id'));
    return act(() =>
      api('/stays', {
        method: 'POST',
        body: { room_id: params.id, user_id },
        token: locals.token,
        fetch
      })
    );
  },
  checkOut: async ({ request, locals, fetch }) => {
    const stayId = String((await request.formData()).get('stay_id'));
    return act(() =>
      api(`/stays/${stayId}/check-out`, { method: 'POST', token: locals.token, fetch })
    );
  }
};
