import type { PageServerLoad } from './$types';
import { api, apiAll } from '$lib/server/api';
import type { AuditEntry, User } from '$lib/api/types';

const PAGE = 100;
const ENTITY_TYPES = ['user', 'room', 'device', 'threshold', 'stay', 'service_call', 'alarm'];

export const load: PageServerLoad = async ({ locals, url, fetch }) => {
  const entityType = url.searchParams.get('entity_type') ?? '';
  const actorId = url.searchParams.get('actor_id') ?? '';
  const offset = Math.max(0, Number(url.searchParams.get('offset') ?? 0) || 0);

  const params = new URLSearchParams({ limit: String(PAGE), offset: String(offset) });
  if (ENTITY_TYPES.includes(entityType)) params.set('entity_type', entityType);
  if (actorId) params.set('actor_id', actorId);

  const [entries, users] = await Promise.all([
    api<AuditEntry[]>(`/audit-log?${params}`, { token: locals.token, fetch }),
    apiAll<User>('/users', { token: locals.token, fetch })
  ]);
  const actors = Object.fromEntries(users.map((u) => [u.id, u.display_name]));
  return {
    entries,
    actors,
    entityType,
    actorId,
    offset,
    pageSize: PAGE,
    hasMore: entries.length === PAGE,
    entityTypes: ENTITY_TYPES
  };
};
