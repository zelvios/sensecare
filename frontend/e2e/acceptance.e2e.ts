import { adminApi, expect, goto, login, test } from './fixtures';

const unique = () => `K-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 5)}`;
const exact = (s: string) => new RegExp(`(^|\\s)${s}(\\s|$)`);

type Seed = {
  room: { id: string; room_number: string };
  device: { id: string; key: string };
};

async function seedRoom(floor = 2): Promise<Seed> {
  const api = await adminApi();
  const room = await (
    await api.post('/rooms', { room_number: unique(), name: 'Acceptance ward', floor })
  ).json();
  const device = await (await api.post('/devices', { room_id: room.id })).json();
  await api.dispose();
  return { room, device };
}

test('K1 / K8: a reading from a device is stored and shown for the right room', async ({
  page,
  staff
}) => {
  const { room, device } = await seedRoom();
  const api = await adminApi();
  const res = await api
    .device(device.id, device.key)
    .post('/devices/measurements', { temperature_c: 22.4, humidity_pct: 41 });
  expect(res.status()).toBe(201);
  const stored = await res.json();
  expect(stored.room_id).toBe(room.id);
  expect(stored.temperature_c).toBe(22.4);
  await api.dispose();

  await login(page, staff);
  await expect(page).toHaveURL('/staff');
  const card = page
    .getByRole('region', { name: 'Rooms' })
    .getByRole('link', { name: exact(room.room_number) });
  await expect(card).toContainText('22.4');
  await expect(card).toContainText('41');
});

test('K2 / K9: a button press appears in the staff panel and can be handled', async ({
  page,
  staff
}) => {
  const { room, device } = await seedRoom();
  const api = await adminApi();
  const pressedAt = new Date();
  const res = await api
    .device(device.id, device.key)
    .post('/devices/service-calls', { pressed_at: pressedAt.toISOString() });
  expect(res.status()).toBe(201);
  const call = await res.json();
  expect(call.room_id).toBe(room.id);
  const latency = (Date.parse(call.created_at) - pressedAt.getTime()) / 1000;
  expect(latency).toBeLessThan(7);
  await api.dispose();

  await login(page, staff);
  await expect(page).toHaveURL('/staff');
  const alerts = page.getByRole('region', { name: 'Attention' });
  await expect(alerts.getByRole('link', { name: exact(room.room_number) })).toBeVisible();

  await goto(page, `/staff/rooms/${room.id}`);
  const calls = page.getByRole('region', { name: 'Service calls' });
  await calls.getByRole('button', { name: 'Acknowledge' }).click();
  await expect(calls.getByRole('listitem').getByText('In progress')).toBeVisible();
  await calls.getByRole('button', { name: 'Close', exact: true }).click();
  await page.getByRole('dialog').getByLabel('Note').fill('Handled during acceptance test');
  await page.getByRole('dialog').getByRole('button', { name: 'Close call' }).click();
  await expect(
    calls.getByRole('listitem').getByText('Closed', { exact: true }).first()
  ).toBeVisible();
  await expect(calls.getByText('Handled during acceptance test')).toBeVisible();
});

test('K3: roles land on their own interface', async ({ page, client, staff, isMobile }) => {
  await login(page, client);
  await expect(page).toHaveURL('/me');
  await expect(page.getByRole('heading', { name: 'My room' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Overview' })).toHaveCount(0);

  if (isMobile) await page.getByRole('button', { name: 'Open menu' }).click();
  await page.getByRole('button', { name: 'Log out' }).click();
  await expect(page).toHaveURL('/login');

  await login(page, staff);
  await expect(page).toHaveURL('/staff');
  await expect(page.getByRole('heading', { name: 'Overview' })).toBeVisible();
});

test('K4: a checked-in client sees their own room, readings and check-in time', async ({
  page,
  client
}) => {
  const { room, device } = await seedRoom();
  const api = await adminApi();
  const users = await (await api.get(`/users?q=${client.username}`)).json();
  const me = users.find((u: { username: string }) => u.username === client.username);
  await api.post('/stays', { room_id: room.id, user_id: me.id });
  await api
    .device(device.id, device.key)
    .post('/devices/measurements', { temperature_c: 23.1, humidity_pct: 44 });
  await api.dispose();

  await login(page, client);
  await expect(page).toHaveURL('/me');
  await expect(page.getByRole('heading', { level: 1 })).toContainText(room.room_number);
  await expect(page.getByText('Checked in')).toBeVisible();
  await expect(page.getByText('23.1 °C').first()).toBeVisible();
  const rows = page.getByRole('table').locator('tbody tr');
  await expect(rows).toHaveCount(1);
  await expect(rows.first()).toContainText('23.1');
});

test('K5 / K6: the overview lists every room and the detail shows history and counts', async ({
  page,
  staff
}) => {
  const a = await seedRoom(1);
  const b = await seedRoom(3);
  const api = await adminApi();
  const node = api.device(a.device.id, a.device.key);
  await node.post('/devices/measurements', { temperature_c: 21, humidity_pct: 40 });
  await node.post('/devices/measurements', { temperature_c: 21.5, humidity_pct: 42 });
  await node.post('/devices/service-calls');
  await api.dispose();

  await login(page, staff);
  await expect(page).toHaveURL('/staff');
  const rooms = page.getByRole('region', { name: 'Rooms' });
  await expect(rooms.getByRole('link', { name: exact(a.room.room_number) })).toBeVisible();
  await expect(rooms.getByRole('link', { name: exact(b.room.room_number) })).toBeVisible();

  await goto(page, `/staff/rooms/${a.room.id}`);
  await expect(page.getByRole('region', { name: 'Service calls' })).toContainText('1');
  const rows = page.getByRole('table').locator('tbody tr');
  await expect(rows).toHaveCount(2);
  await page.getByRole('link', { name: '7 days' }).click();
  await expect(page).toHaveURL(/hours=168/);
  await expect(rows).toHaveCount(2);
});

test('K7: an admin manages rooms and accounts, and a deactivated account cannot log in', async ({
  page,
  browser
}) => {
  const api = await adminApi();
  const suffix = unique();
  const account = { username: `k7-${suffix}`.toLowerCase(), password: `k7-pass-${suffix}-12` };
  await api.post('/users', { ...account, display_name: 'K7 Nurse', role: 'staff' });
  await api.dispose();

  await login(page, {
    username: 'admin',
    password: process.env.E2E_ADMIN_PASSWORD ?? 'admin-change-me'
  });
  await expect(page).toHaveURL('/staff');
  await goto(page, '/admin/rooms');
  await page.getByRole('button', { name: 'New room' }).click();
  const dialog = page.getByRole('dialog');
  await dialog.getByLabel('Room number').fill(suffix);
  await dialog.getByLabel('Name').fill('K7 room');
  await dialog.getByRole('button', { name: 'Save' }).click();
  const row = page.getByRole('row', { name: exact(suffix) });
  await expect(row).toContainText('K7 room');
  await row.getByRole('button', { name: 'Deactivate' }).click();
  await expect(row).toContainText('Deactivated');

  await goto(page, '/admin/accounts');
  await page.getByLabel('Search name or username').fill(account.username);
  const userRow = page.getByRole('row', { name: new RegExp(account.username) });
  await userRow.getByRole('button', { name: 'Deactivate' }).click();
  await expect(userRow).toContainText('Deactivated');

  const other = await browser.newPage();
  await login(other, account);
  await expect(other).toHaveURL('/login');
  await expect(other.getByRole('alert')).toBeVisible();
  await other.close();
});

test('K10: two devices report to their own rooms, and a bad key is refused', async () => {
  const a = await seedRoom();
  const b = await seedRoom();
  const api = await adminApi();
  const ra = await (
    await api
      .device(a.device.id, a.device.key)
      .post('/devices/measurements', { temperature_c: 20, humidity_pct: 40 })
  ).json();
  const rb = await (
    await api
      .device(b.device.id, b.device.key)
      .post('/devices/measurements', { temperature_c: 25, humidity_pct: 50 })
  ).json();
  expect(ra.room_id).toBe(a.room.id);
  expect(rb.room_id).toBe(b.room.id);

  const bad = await api
    .device(a.device.id, 'f'.repeat(64))
    .post('/devices/measurements', { temperature_c: 20, humidity_pct: 40 });
  expect(bad.status()).toBe(401);
  await api.dispose();
});

test('K11: a client cannot open staff pages or read another room', async ({ page, client }) => {
  const other = await seedRoom();
  await login(page, client);
  await expect(page).toHaveURL('/me');

  const staffPage = await page.goto('/staff');
  expect(staffPage?.status()).toBe(403);
  const adminPage = await page.goto('/admin');
  expect(adminPage?.status()).toBe(403);

  const api = await adminApi();
  const clientLogin = await api.post('/auth/login', client);
  const token = (await clientLogin.json()).session_token as string;
  const res = await api.getAs(token, `/rooms/${other.room.id}/measurements`);
  expect(res.status()).toBe(403);
  await api.dispose();
});

test('K12: the session cookie is HttpOnly and never reaches page scripts', async ({
  page,
  context,
  client
}) => {
  await login(page, client);
  await expect(page).toHaveURL('/me');
  const cookie = (await context.cookies()).find((c) => c.name === 'sensecare_session');
  expect(cookie?.httpOnly).toBe(true);
  expect(cookie?.sameSite).toBe('Lax');
  expect(await page.evaluate(() => document.cookie)).not.toContain('sensecare_session');
});

test.fixme('K12: HTTP is redirected to HTTPS', async () => {
  // Verified on the deployed server
});

test.fixme('K13: the system starts with one command and keeps data across restarts', async () => {
  // Verified with docker compose on the server
});
