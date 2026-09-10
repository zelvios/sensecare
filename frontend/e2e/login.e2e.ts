import { expect, goto, login, logout, test } from './fixtures';

test('K3: a client lands on their room, a staff member on the overview', async ({
	page,
	client,
	staff,
	isMobile
}) => {
	await login(page, client);
	await expect(page).toHaveURL('/me');
	await expect(page.getByRole('heading', { name: 'My room' })).toBeVisible();

	await logout(page, isMobile);

	await login(page, staff);
	await expect(page).toHaveURL('/staff');
	await expect(page.getByRole('heading', { name: 'Overview' })).toBeVisible();
});

test('wrong password shows an error and stays on the login page', async ({ page, client }) => {
	await login(page, { username: client.username, password: 'not-the-password-1' });
	await expect(page).toHaveURL('/login');
	await expect(page.getByRole('alert')).toHaveText('Wrong username or password');
	await expect(page.getByLabel('Username')).toHaveValue(client.username);
});

test('K11: a client cannot open the staff pages', async ({ page, client }) => {
	await login(page, client);
	await expect(page).toHaveURL('/me');
	const res = await page.goto('/staff');
	expect(res?.status()).toBe(403);
});

test('logged out visitors are sent to login', async ({ page }) => {
	await page.goto('/staff');
	await expect(page).toHaveURL(/\/login\?next=%2Fstaff/);
});

test('the session cookie is HttpOnly', async ({ page, context, client }) => {
	await login(page, client);
	await expect(page).toHaveURL('/me');
	const cookie = (await context.cookies()).find((c) => c.name === 'sensecare_session');
	expect(cookie?.httpOnly).toBe(true);
	expect(cookie?.sameSite).toBe('Lax');
	expect(await page.evaluate(() => document.cookie)).not.toContain('sensecare_session');
});

test('language switch changes the interface', async ({ page }) => {
	await goto(page, '/login');
	await page.getByLabel('Language').selectOption('da');
	await expect(page.getByRole('button', { name: 'Log ind' })).toBeVisible();
	await page.getByLabel('Sprog').selectOption('en');
	await expect(page.getByRole('button', { name: 'Log in' })).toBeVisible();
});
