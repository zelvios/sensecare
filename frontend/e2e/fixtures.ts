import { expect, request, test as base, type Page } from '@playwright/test';

const API = process.env.E2E_API_URL ?? 'http://localhost:8080';
export const ADMIN = {
	username: 'admin',
	password: process.env.E2E_ADMIN_PASSWORD ?? 'admin-change-me'
};

export type Account = { username: string; password: string; display_name: string };

/** Creates a fresh account through the API. Unique name per run so tests can run in parallel. */
async function createAccount(role: 'client' | 'staff'): Promise<Account> {
	const api = await request.newContext({ baseURL: API });
	const login = await api.post('/api/v1/auth/login', { data: ADMIN });
	const token = (await login.json()).session_token as string;

	const suffix = Math.random().toString(36).slice(2, 8);
	const account = {
		username: `e2e-${role}-${suffix}`,
		password: `e2e-pass-${suffix}-12`,
		display_name: role === 'client' ? 'Test Patient' : 'Test Nurse'
	};
	const res = await api.post('/api/v1/users', {
		headers: { authorization: `Bearer ${token}` },
		data: { ...account, role }
	});
	if (!res.ok()) throw new Error(`could not create ${role}: ${res.status()}`);
	await api.dispose();
	return account;
}

/**
 * Navigates and waits until the page is interactive. `page.goto` returns when the
 * HTML has loaded, but SvelteKit's JavaScript which attaches the click and change
 * handlers, keeps loading for a moment after that under `vite dev`.
 */
export async function goto(page: Page, path: string) {
	await page.goto(path);
	await page.waitForLoadState('networkidle');
}

/** Fills in the login form. */
export async function login(page: Page, account: Pick<Account, 'username' | 'password'>) {
	await goto(page, '/login');
	await page.getByLabel('Username').fill(account.username);
	await page.getByLabel('Password').fill(account.password);
	await page.getByRole('button', { name: 'Log in' }).click();
}

/** Logs out from wherever the button lives: the header on desktop, the drawer on phones. */
export async function logout(page: Page, isMobile: boolean) {
	if (isMobile) await page.getByRole('button', { name: 'Open menu' }).click();
	await page.getByRole('button', { name: 'Log out' }).click();
	await expect(page).toHaveURL('/login');
}

export const test = base.extend<{ client: Account; staff: Account }>({
	// eslint-disable-next-line no-empty-pattern
	client: async ({}, use) => use(await createAccount('client')),
	// eslint-disable-next-line no-empty-pattern
	staff: async ({}, use) => use(await createAccount('staff'))
});

export { expect } from '@playwright/test';
