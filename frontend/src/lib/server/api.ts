// The only place that talks to the API. Server-side only.
import { env } from '$env/dynamic/private';
import type { ErrorResponse } from '$lib/api/types';

const BASE = (env.API_URL ?? 'http://localhost:8080').replace(/\/$/, '');

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly code: string,
    message: string
  ) {
    super(message);
  }
}

type Options = {
  method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  token?: string | null;
  body?: unknown;
  fetch?: typeof fetch;
};

/** Calls the API and returns the parsed JSON, or throws ApiError with the API's error code. */
export async function api<T>(path: string, opts: Options = {}): Promise<T> {
  const headers: Record<string, string> = { accept: 'application/json' };
  if (opts.token) headers.authorization = `Bearer ${opts.token}`;
  if (opts.body !== undefined) headers['content-type'] = 'application/json';

  const res = await (opts.fetch ?? fetch)(`${BASE}/api/v1${path}`, {
    method: opts.method ?? 'GET',
    headers,
    body: opts.body === undefined ? undefined : JSON.stringify(opts.body)
  });

  if (res.status === 204) return undefined as T;

  const data = await res.json().catch(() => null);
  if (!res.ok) {
    const err = data as ErrorResponse | null;
    throw new ApiError(res.status, err?.error ?? 'unknown', err?.message ?? res.statusText);
  }
  return data as T;
}
