import type { Actions, PageServerLoad } from './$types';
import { api } from '$lib/server/api';
import { act } from '$lib/server/actions';
import type { AuditEntry, User } from '$lib/api/types';

export const load: PageServerLoad = async ({ locals, params, fetch }) => {
  const token = locals.token;
  const [account, about, by] = await Promise.all([
    api<User>(`/users/${params.id}`, { token, fetch }),
    api<AuditEntry[]>(`/audit-log/user/${params.id}`, { token, fetch }),
    api<AuditEntry[]>(`/audit-log?actor_id=${params.id}&limit=100`, { token, fetch })
  ]);
  return { account, about, by };
};

export const actions: Actions = {
  update: async ({ request, locals, params, fetch }) => {
    const form = await request.formData();
    const body = {
      username: String(form.get('username') ?? '').trim(),
      display_name: String(form.get('display_name') ?? '').trim(),
      role: String(form.get('role') ?? '')
    };
    return act(() =>
      api(`/users/${params.id}`, { method: 'PATCH', body, token: locals.token, fetch })
    );
  },
  password: async ({ request, locals, params, fetch }) => {
    const password = String((await request.formData()).get('password') ?? '');
    return act(() =>
      api(`/users/${params.id}/password`, {
        method: 'POST',
        body: { password },
        token: locals.token,
        fetch
      })
    );
  },
  deactivate: async ({ locals, params, fetch }) =>
    act(() =>
      api(`/users/${params.id}/deactivate`, { method: 'POST', token: locals.token, fetch })
    ),
  activate: async ({ locals, params, fetch }) =>
    act(() => api(`/users/${params.id}/activate`, { method: 'POST', token: locals.token, fetch }))
};
