import type { Actions, PageServerLoad } from './$types';
import { api, apiAll } from '$lib/server/api';
import { act, actWith } from '$lib/server/actions';
import type { Device, DeviceWithKey, Room } from '$lib/api/types';

export const load: PageServerLoad = async ({ locals, fetch }) => {
  const token = locals.token;
  const [devices, rooms] = await Promise.all([
    apiAll<Device>('/devices', { token, fetch }),
    apiAll<Room>('/rooms', { token, fetch })
  ]);
  return { devices, rooms, loadedAt: new Date().toISOString() };
};

function deviceBody(form: FormData) {
  const label = String(form.get('label') ?? '').trim();
  const firmware = String(form.get('firmware_version') ?? '').trim();
  return { label: label || null, firmware_version: firmware || null };
}

export const actions: Actions = {
  register: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    const roomId = String(form.get('room_id') ?? '');
    const body = { ...deviceBody(form), room_id: roomId || null };
    return actWith(() =>
      api<DeviceWithKey>('/devices', { method: 'POST', body, token: locals.token, fetch })
    );
  },
  update: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    const id = String(form.get('id'));
    return act(() =>
      api(`/devices/${id}`, { method: 'PATCH', body: deviceBody(form), token: locals.token, fetch })
    );
  },
  assign: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    const id = String(form.get('id'));
    const roomId = String(form.get('room_id') ?? '');
    return act(() =>
      api(`/devices/${id}/assign`, {
        method: 'POST',
        body: { room_id: roomId || null },
        token: locals.token,
        fetch
      })
    );
  },
  rotateKey: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return actWith(() =>
      api<DeviceWithKey>(`/devices/${id}/rotate-key`, {
        method: 'POST',
        token: locals.token,
        fetch
      })
    );
  },
  deactivate: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() =>
      api(`/devices/${id}/deactivate`, { method: 'POST', token: locals.token, fetch })
    );
  },
  activate: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() =>
      api(`/devices/${id}/activate`, { method: 'POST', token: locals.token, fetch })
    );
  },
  delete: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() => api(`/devices/${id}`, { method: 'DELETE', token: locals.token, fetch }));
  }
};
