import type { PageServerLoad } from './$types';
import { api, ApiError } from '$lib/server/api';
import {
  type Measurement,
  PERIOD_HOURS,
  type PeriodHours,
  type Stay,
  type Threshold
} from '$lib/api/types';

/** Turns a 404 into null. Everything else still throws. */
async function orNull<T>(p: Promise<T>): Promise<T | null> {
  try {
    return await p;
  } catch (e) {
    if (e instanceof ApiError && e.status === 404) return null;
    throw e;
  }
}

export const load: PageServerLoad = async ({ locals, url, fetch }) => {
  const token = locals.token;
  const requested = Number(url.searchParams.get('hours'));
  const hours: PeriodHours = PERIOD_HOURS.includes(requested as PeriodHours)
    ? (requested as PeriodHours)
    : 24;

  const stay = await orNull(api<Stay>('/stays/me', { token, fetch }));
  if (!stay) return { stay: null, hours, latest: null, thresholds: null, history: [] };

  const to = new Date();
  const from = new Date(to.getTime() - hours * 3600 * 1000);
  const room = stay.room_id;

  const [latest, thresholds, history] = await Promise.all([
    orNull(api<Measurement>(`/rooms/${room}/measurements/latest`, { token, fetch })),
    api<Threshold>(`/rooms/${room}/thresholds`, { token, fetch }),
    api<Measurement[]>(
      `/rooms/${room}/measurements?from=${from.toISOString()}&to=${to.toISOString()}&limit=500`,
      { token, fetch }
    )
  ]);

  return { stay, hours, latest, thresholds, history };
};
