import { redirect } from '@sveltejs/kit';
import type { Actions } from './$types';
import { api } from '$lib/server/api';
import { clearSession } from '$lib/server/session';

export const actions: Actions = {
  default: async ({ locals, cookies, fetch }) => {
    if (locals.token) {
      await api('/auth/logout', { method: 'POST', token: locals.token, fetch }).catch(() => {});
    }
    clearSession(cookies);
    redirect(303, '/login');
  }
};
