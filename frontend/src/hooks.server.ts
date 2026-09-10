// Runs before every request. Turns the session cookie into event.locals.user
// by asking the API, so pages never trust the cookie alone.
import type { Handle } from '@sveltejs/kit';
import { api, ApiError } from '$lib/server/api';
import { clearSession, SESSION_COOKIE } from '$lib/server/session';
import type { AuthenticatedUser } from '$lib/api/types';

export const handle: Handle = async ({ event, resolve }) => {
	const token = event.cookies.get(SESSION_COOKIE) ?? null;
	event.locals.token = token;
	event.locals.user = null;

	if (token) {
		try {
			event.locals.user = await api<AuthenticatedUser>('/auth/me', { token, fetch: event.fetch });
		} catch (e) {
			// Expired, logged out elsewhere, or account deactivated: drop the cookie.
			if (e instanceof ApiError && (e.status === 401 || e.status === 403)) {
				clearSession(event.cookies);
				event.locals.token = null;
			} else {
				throw e;
			}
		}
	}

	const response = await resolve(event);
	response.headers.set('x-content-type-options', 'nosniff');
	response.headers.set('x-frame-options', 'DENY');
	response.headers.set('referrer-policy', 'strict-origin-when-cross-origin');
	return response;
};
