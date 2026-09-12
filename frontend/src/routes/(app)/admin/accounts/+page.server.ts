import type { Actions, PageServerLoad } from './$types';
import { api } from '$lib/server/api';
import { act } from '$lib/server/actions';
import type { User } from '$lib/api/types';

export const load: PageServerLoad = async ({ locals, fetch }) => {
  const users = await api<User[]>('/users?limit=200', { token: locals.token, fetch });
  return { users };
};

export const actions: Actions = {
  create: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    const body = {
      username: String(form.get('username') ?? '').trim(),
      display_name: String(form.get('display_name') ?? '').trim(),
      password: String(form.get('password') ?? ''),
      role: String(form.get('role') ?? 'client')
    };
    return act(() => api('/users', { method: 'POST', body, token: locals.token, fetch }));
  },
  update: async ({ request, locals, fetch }) => {
    const form = await request.formData();
    const id = String(form.get('id'));
    const body = {
      username: String(form.get('username') ?? '').trim(),
      display_name: String(form.get('display_name') ?? '').trim(),
      role: String(form.get('role') ?? '')
    };
    return act(() => api(`/users/${id}`, { method: 'PATCH', body, token: locals.token, fetch }));
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
  delete: async ({ request, locals, fetch }) => {
    const id = String((await request.formData()).get('id'));
    return act(() => api(`/users/${id}`, { method: 'DELETE', token: locals.token, fetch }));
  }
};
