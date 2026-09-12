import type { components } from './schema';

type S = components['schemas'];

export type AuthenticatedUser = S['AuthenticatedUser'];
export type LoginResponse = S['LoginResponse'];
export type ErrorResponse = S['ErrorResponse'];
export type Role = AuthenticatedUser['role'];

export type Stay = S['StayResponse'];
export type Measurement = S['MeasurementResponse'];
export type Threshold = S['ThresholdResponse'];

export type RoomOverview = S['RoomOverviewResponse'];
export type OpenCall = RoomOverview['open_calls'][number];
export type OpenAlarm = RoomOverview['open_alarms'][number];

export type ServiceCall = S['ServiceCallResponse'];
export type Alarm = S['AlarmResponse'];

export type Room = S['Room'];

/** History periods offered in the UI, in hours. */
export const PERIOD_HOURS = [24, 3 * 24, 7 * 24] as const;
export type PeriodHours = (typeof PERIOD_HOURS)[number];
