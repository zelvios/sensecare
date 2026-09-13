import type { Actions, PageServerLoad } from './$types';
import { api } from '$lib/server/api';
import { act } from '$lib/server/actions';
import type { Alarm } from '$lib/api/types';

export const load: PageServerLoad = async ({ locals, fetch }) => {
  const alarms = await api<Alarm[]>('/alarms?limit=200', { token: locals.token, fetch });
  return { alarms, loadedAt: new Date().toISOString() };
};

export const actions: Actions = {
  acknowledge: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() =>
      api(`/alarms/${id}/acknowledge`, { method: 'POST', token: locals.token, fetch })
    );
  },
  resolve: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() => api(`/alarms/${id}/resolve`, { method: 'POST', token: locals.token, fetch }));
  }
};
