import type { Actions, PageServerLoad } from './$types';
import { api, apiAll } from '$lib/server/api';
import { act } from '$lib/server/actions';
import type { Room, Stay, User } from '$lib/api/types';

export const load: PageServerLoad = async ({ locals, fetch }) => {
  const token = locals.token;
  const [patients, stays, rooms] = await Promise.all([
    apiAll<User>('/users?role=client', { token, fetch }),
    apiAll<Stay>('/stays?open=true', { token, fetch }),
    apiAll<Room>('/rooms?active=true', { token, fetch })
  ]);
  const stayByUser = Object.fromEntries(stays.map((s) => [s.user_id, s]));
  const occupiedRoomIds = new Set(stays.map((s) => s.room_id));
  const freeRooms = rooms.filter((r) => !occupiedRoomIds.has(r.id));
  return { patients, stayByUser, freeRooms };
};

export const actions: Actions = {
  create: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    const body = {
      username: String(form.get('username') ?? '').trim(),
      display_name: String(form.get('display_name') ?? '').trim(),
      password: String(form.get('password') ?? ''),
      role: 'client'
    };
    return act(() => api('/users', { method: 'POST', body, token: locals.token, fetch }));
  },
  update: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    const id = String(form.get('id'));
    const body = {
      username: String(form.get('username') ?? '').trim(),
      display_name: String(form.get('display_name') ?? '').trim()
    };
    return act(() => api(`/users/${id}`, { method: 'PATCH', body, token: locals.token, fetch }));
  },
  password: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    const id = String(form.get('id'));
    const password = String(form.get('password') ?? '');
    return act(() =>
      api(`/users/${id}/password`, {
        method: 'POST',
        body: { password },
        token: locals.token,
        fetch
      })
    );
  },
  deactivate: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() =>
      api(`/users/${id}/deactivate`, { method: 'POST', token: locals.token, fetch })
    );
  },
  activate: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() => api(`/users/${id}/activate`, { method: 'POST', token: locals.token, fetch }));
  },
  checkIn: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    const body = { user_id: String(form.get('user_id')), room_id: String(form.get('room_id')) };
    return act(() => api('/stays', { method: 'POST', body, token: locals.token, fetch }));
  },
  checkOut: async ({ request, locals, fetch }) => {
    const stayId = String((await request.formData()).get('stay_id'));
    return act(() =>
      api(`/stays/${stayId}/check-out`, { method: 'POST', token: locals.token, fetch })
    );
  }
};
