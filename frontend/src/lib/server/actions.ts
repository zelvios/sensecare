import { fail } from '@sveltejs/kit';
import { ApiError } from '$lib/server/api';

/**
 * Runs an API call inside a form action. Returns `{ ok: true }` on success, or
 * `fail(status, { error: code })` so the page can show a translated message.
 */
export async function act(fn: () => Promise<unknown>) {
  try {
    await fn();
    return { ok: true };
  } catch (e) {
    if (e instanceof ApiError) return fail(e.status, { error: e.code });
    throw e;
  }
}

/** Like act, but returns the API response so the page can show it (e.g. a device key). */
export async function actWith<T>(fn: () => Promise<T>) {
  try {
    return { ok: true, data: await fn() };
  } catch (e) {
    if (e instanceof ApiError) return fail(e.status, { error: e.code });
    throw e;
  }
}
