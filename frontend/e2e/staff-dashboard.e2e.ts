import { adminApi, expect, goto, login, test } from './fixtures';

/** A room number that is unique per run and short enough for the 16-character column. */
const uniqueNumber = () => `E2E-${Date.now().toString(36)}`;

/** Matches the room number as a whole word, so E2E-187 does not also match E2E-1870. */
const exact = (number: string) => new RegExp(`(^|\\s)${number}(\\s|$)`);

/** A room on floor 2 with a device, one hot reading (raises an alarm) and one button press. */
async function seedRoom(number: string) {
  const api = await adminApi();
  const room = await (
    await api.post('/rooms', { room_number: number, name: 'E2E ward', floor: 2 })
  ).json();
  const dev = await (await api.post('/devices', { room_id: room.id })).json();
  const node = api.device(dev.id, dev.key);
  await node.post('/devices/measurements', { temperature_c: 30.0, humidity_pct: 45.0 });
  await node.post('/devices/service-calls');
  await api.dispose();
  return room;
}

test('K5: the overview lists the room with its reading, call and alarm', async ({
  page,
  staff
}) => {
  const number = uniqueNumber();
  await seedRoom(number);

  await login(page, staff);
  await expect(page).toHaveURL('/staff');

  const rooms = page.getByRole('region', { name: 'Rooms' });
  const card = rooms.getByRole('link', { name: exact(number) });
  await expect(card).toBeVisible();
  await expect(card).toContainText(`Floor 2, ${number}`);
  await expect(card).toContainText('30');
  await expect(card.getByLabel('Service call')).toBeVisible();

  const alerts = page.getByRole('region', { name: 'Attention' });
  const mine = alerts.getByRole('link', { name: exact(number) });
  await expect(mine).toHaveCount(2);
  await expect(mine.filter({ hasText: 'Temperature too high' })).toBeVisible();
  await expect(mine.filter({ hasText: 'Service call' })).toBeVisible();
});

test('K5 / K6: the room detail shows counts and history', async ({ page, staff }) => {
  const number = uniqueNumber();
  const room = await seedRoom(number);

  await login(page, staff);
  await expect(page).toHaveURL('/staff');
  await goto(page, `/staff/rooms/${room.id}`);

  await expect(page.getByRole('heading', { level: 1 })).toContainText(number);
  await expect(page.getByRole('region', { name: 'Alarms' })).toContainText('1');
  await expect(page.getByRole('region', { name: 'Service calls' })).toContainText('1');
  await expect(page.getByText('the global default')).toBeVisible();

  const rows = page.getByRole('table').locator('tbody tr');
  await expect(rows).toHaveCount(1);
  await expect(rows.first()).toContainText('30');

  await page.getByRole('link', { name: '7 days' }).click();
  await expect(page).toHaveURL(/hours=168/);
  await expect(rows).toHaveCount(1);
});

test('the overview search narrows to the typed floor', async ({ page, staff }) => {
  const number = uniqueNumber();
  await seedRoom(number);

  await login(page, staff);
  await expect(page).toHaveURL('/staff');
  await goto(page, '/staff');

  const rooms = page.getByRole('region', { name: 'Rooms' });
  const search = page.getByLabel('Search floor, number or name');

  await search.fill('floor 2');
  await expect(rooms.getByRole('link', { name: exact(number) })).toBeVisible();
  await search.fill('floor 99');
  await expect(page.getByText('No rooms match the search.')).toBeVisible();
});
