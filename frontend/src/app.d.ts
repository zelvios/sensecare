import type { AuthenticatedUser } from '$lib/api/types';

declare global {
  namespace App {
    interface Locals {
      user: AuthenticatedUser | null;
      token: string | null;
    }
  }
}

export {};
