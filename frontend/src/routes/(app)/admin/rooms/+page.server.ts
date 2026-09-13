import type { Actions, PageServerLoad } from './$types';
import { api } from '$lib/server/api';
import { act } from '$lib/server/actions';
import type { Room, Stay, User } from '$lib/api/types';

export const load: PageServerLoad = async ({ locals, fetch }) => {
  const token = locals.token;
  const [rooms, stays, clients] = await Promise.all([
    api<Room[]>('/rooms?limit=200', { token, fetch }),
    api<Stay[]>('/stays?open=true&limit=200', { token, fetch }),
    api<User[]>('/users?role=client&active=true&limit=200', { token, fetch })
  ]);
  const staysByRoom = Object.fromEntries(stays.map((s) => [s.room_id, s]));
  const occupiedIn = Object.fromEntries(stays.map((s) => [s.user_id, s.room_number]));
  return { rooms, staysByRoom, clients, occupiedIn };
};

function roomBody(form: FormData) {
  const name = String(form.get('name') ?? '').trim();
  const floorRaw = String(form.get('floor') ?? '').trim();
  return {
    room_number: String(form.get('room_number') ?? '').trim(),
    name: name || null,
    floor: floorRaw === '' ? null : Number(floorRaw)
  };
}

export const actions: Actions = {
  create: async ({ request, locals, fetch }) => {
    const body = roomBody(await request.formData());
    return act(() => api('/rooms', { method: 'POST', body, token: locals.token, fetch }));
  },
  update: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    const id = String(form.get('id'));
    const body = roomBody(form);
    return act(() => api(`/rooms/${id}`, { method: 'PATCH', body, token: locals.token, fetch }));
  },
  deactivate: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() =>
      api(`/rooms/${id}/deactivate`, { method: 'POST', token: locals.token, fetch })
    );
  },
  activate: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() => api(`/rooms/${id}/activate`, { method: 'POST', token: locals.token, fetch }));
  },
  delete: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() => api(`/rooms/${id}`, { method: 'DELETE', token: locals.token, fetch }));
  },
  checkIn: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    const body = { room_id: String(form.get('room_id')), user_id: String(form.get('user_id')) };
    return act(() => api('/stays', { method: 'POST', body, token: locals.token, fetch }));
  },
  checkOut: async ({ request, locals, fetch }) => {
    const stayId = String((await request.formData()).get('stay_id'));
    return act(() =>
      api(`/stays/${stayId}/check-out`, { method: 'POST', token: locals.token, fetch })
    );
  }
};
