import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { api, ApiError } from '$lib/server/api';
import { setSession } from '$lib/server/session';
import type { LoginResponse } from '$lib/api/types';

export const load: PageServerLoad = ({ locals }) => {
  if (locals.user) redirect(303, '/');
};

export const actions: Actions = {
  default: async ({ request, cookies, fetch }) => {
    const form = await request.formData();
    const username = String(form.get('username') ?? '').trim();
    const password = String(form.get('password') ?? '');
    if (!username || !password)
      return fail(400, { username, error: 'Udfyld brugernavn og adgangskode' });

    try {
      const res = await api<LoginResponse>('/auth/login', {
        method: 'POST',
        body: { username, password },
        fetch
      });
      setSession(cookies, res.session_token);
      redirect(303, res.user.role === 'client' ? '/me' : '/staff');
    } catch (e) {
      if (e instanceof ApiError) {
        const error =
          e.code === 'account_deactivated'
            ? 'Kontoen er deaktiveret'
            : 'Forkert brugernavn eller adgangskode';
        return fail(e.status, { username, error });
      }
      throw e;
    }
  }
};
