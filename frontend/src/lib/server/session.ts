import type { Cookies } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';

export const SESSION_COOKIE = 'sensecare_session';

const ttlHours = Number(env.SESSION_TTL_HOURS ?? 12);

export function setSession(cookies: Cookies, token: string) {
  cookies.set(SESSION_COOKIE, token, {
    path: '/',
    httpOnly: true,
    sameSite: 'lax',
    secure: process.env.NODE_ENV === 'production',
    maxAge: ttlHours * 60 * 60
  });
}

export function clearSession(cookies: Cookies) {
  cookies.delete(SESSION_COOKIE, { path: '/' });
}
