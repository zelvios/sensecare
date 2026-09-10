import type { components } from './schema';

type S = components['schemas'];

export type AuthenticatedUser = S['AuthenticatedUser'];
export type LoginResponse = S['LoginResponse'];
export type ErrorResponse = S['ErrorResponse'];
export type Role = AuthenticatedUser['role'];

export type Stay = S['StayResponse'];
export type Measurement = S['MeasurementResponse'];
export type Threshold = S['ThresholdResponse'];

/** History periods offered in the UI, in hours. */
export const PERIOD_HOURS = [24, 72, 168] as const;
export type PeriodHours = (typeof PERIOD_HOURS)[number];
