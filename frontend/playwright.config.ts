import process from 'node:process';
import {defineConfig, devices} from '@playwright/test';

try {
  process.loadEnvFile('.env');
} catch {
  // No .env file, the defaults in fixtures.ts apply.
}

export default defineConfig({
  testDir: 'e2e',
  testMatch: '**/*.e2e.ts',
  fullyParallel: true,
  workers: process.env.CI ? undefined : 4,
  timeout: 60_000,
  retries: process.env.CI ? 2 : 0,
  reporter: process.env.CI ? [['list'], ['html', { open: 'never' }]] : 'list',
  expect: { timeout: 10_000 },
  use: {
    baseURL: 'http://localhost:5173',
    locale: 'en-US',
    trace: 'retain-on-failure'
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] }, testIgnore: /phone-menu/ },
    { name: 'phone', use: { ...devices['Pixel 7'] } }
  ],
  webServer: {
    command: 'vite dev',
    url: 'http://localhost:5173/healthz',
    reuseExistingServer: true,
    timeout: 120_000
  }
});
