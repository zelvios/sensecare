import type { Actions, PageServerLoad } from './$types';
import { api } from '$lib/server/api';
import { act } from '$lib/server/actions';
import type { ServiceCall } from '$lib/api/types';

export const load: PageServerLoad = async ({ locals, fetch }) => {
  const calls = await api<ServiceCall[]>('/service-calls?limit=200', {
    token: locals.token,
    fetch
  });
  return { calls, loadedAt: new Date().toISOString() };
};

export const actions: Actions = {
  simulate: async ({ request, fetch }) => {
    const form = await request.formData();
    const id = String(form.get('device_id') ?? '').trim();
    const key = String(form.get('device_key') ?? '').trim();
    return act(() =>
      api('/devices/service-calls', {
        method: 'POST',
        headers: { 'x-device-id': id, 'x-device-key': key },
        fetch
      })
    );
  },
  acknowledge: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() =>
      api(`/service-calls/${id}/acknowledge`, { method: 'POST', token: locals.token, fetch })
    );
  },
  close: async ({ request, locals, fetch }) => {
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
  }
};
