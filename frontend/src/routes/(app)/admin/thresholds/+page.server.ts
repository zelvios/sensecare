import type { Actions, PageServerLoad } from './$types';
import { api } from '$lib/server/api';
import { act } from '$lib/server/actions';
import type { Room, Threshold } from '$lib/api/types';

export const load: PageServerLoad = async ({ locals, fetch }) => {
  const token = locals.token;
  const [global, overrides, rooms] = await Promise.all([
    api<Threshold>('/thresholds', { token, fetch }),
    api<Threshold[]>('/thresholds/rooms', { token, fetch }),
    api<Room[]>('/rooms?limit=200', { token, fetch })
  ]);
  return { global, overrides, rooms };
};

function limits(form: FormData) {
  const num = (k: string) => Number(String(form.get(k) ?? '').replace(',', '.'));
  return {
    temperature_min: num('temperature_min'),
    temperature_max: num('temperature_max'),
    humidity_min: num('humidity_min'),
    humidity_max: num('humidity_max')
  };
}

export const actions: Actions = {
  create: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    const roomId = String(form.get('room_id') ?? '');
    return act(() =>
      api(`/rooms/${roomId}/thresholds`, {
        method: 'PUT',
        body: limits(form),
        token: locals.token,
        fetch
      })
    );
  },
  update: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    const roomId = String(form.get('room_id') ?? '');
    return act(() =>
      api(`/rooms/${roomId}/thresholds`, {
        method: 'PUT',
        body: limits(form),
        token: locals.token,
        fetch
      })
    );
  },
  updateGlobal: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    return act(() =>
      api('/thresholds', { method: 'PUT', body: limits(form), token: locals.token, fetch })
    );
  },
  delete: async ({ request, locals, fetch }) => {
    const roomId = String((await request.formData()).get('room_id') ?? '');
    return act(() =>
      api(`/rooms/${roomId}/thresholds`, { method: 'DELETE', token: locals.token, fetch })
    );
  }
};
