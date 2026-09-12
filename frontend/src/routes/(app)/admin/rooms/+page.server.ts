import type { Actions, PageServerLoad } from './$types';
import { api } from '$lib/server/api';
import { act } from '$lib/server/actions';
import type { Room } from '$lib/api/types';

export const load: PageServerLoad = async ({ locals, fetch }) => {
  const rooms = await api<Room[]>('/rooms?limit=200', { token: locals.token, fetch });
  rooms.sort((a, b) => {
    const fa = a.floor ?? Number.POSITIVE_INFINITY;
    const fb = b.floor ?? Number.POSITIVE_INFINITY;
    if (fa !== fb) return fa - fb;
    return a.room_number.localeCompare(b.room_number, undefined, { numeric: true });
  });
  return { rooms };
};

/** Reads the room fields from a form. Empty name and floor become null. */
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
  }
};
