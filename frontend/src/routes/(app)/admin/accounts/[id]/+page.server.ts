import type { Actions, PageServerLoad } from './$types';
import { api, apiAll } from '$lib/server/api';
import { act } from '$lib/server/actions';
import type { AuditEntry, Stay, User } from '$lib/api/types';

export const load: PageServerLoad = async ({ locals, params, fetch }) => {
  const token = locals.token;
  const [account, about, by, users, stays] = await Promise.all([
    api<User>(`/users/${params.id}`, { token, fetch }),
    api<AuditEntry[]>(`/audit-log/user/${params.id}`, { token, fetch }),
    api<AuditEntry[]>(`/audit-log?actor_id=${params.id}&limit=100`, { token, fetch }),
    apiAll<User>('/users', { token, fetch }),
    api<Stay[]>(`/stays?user_id=${params.id}&limit=50`, { token, fetch })
  ]);
  return {
    account,
    about,
    by,
    actors: Object.fromEntries(users.map((u) => [u.id, u.display_name])),
    stays
  };
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
