import type { components } from './schema';

type S = components['schemas'];

export type AuthenticatedUser = S['AuthenticatedUser'];
export type LoginResponse = S['LoginResponse'];
export type ErrorResponse = S['ErrorResponse'];
export type Role = AuthenticatedUser['role'];
