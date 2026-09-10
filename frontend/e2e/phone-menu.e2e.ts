import { expect, login, test } from './fixtures';

test.describe('phone', () => {
  test.skip(({ isMobile }) => !isMobile, 'phone layout only');

  test('the burger menu shows navigation and logout', async ({ page, client }) => {
    await login(page, client);
    await expect(page).toHaveURL('/me');

    await page.getByRole('button', { name: 'Open menu' }).click();
    const menu = page.getByRole('dialog');
    await expect(menu).toBeVisible();
    await expect(menu.getByRole('link', { name: 'My room' })).toBeVisible();
    await expect(menu.getByText(client.display_name)).toBeVisible();

    await page.keyboard.press('Escape');
    await expect(menu).toBeHidden();
  });
});
