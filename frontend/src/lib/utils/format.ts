import { getLocale } from '$lib/paraglide/runtime';

export function fmtDateTime(iso: string): string {
  return new Intl.DateTimeFormat(getLocale(), { dateStyle: 'short', timeStyle: 'short' }).format(
    new Date(iso)
  );
}

export function fmtTime(iso: string): string {
  return new Intl.DateTimeFormat(getLocale(), { timeStyle: 'short' }).format(new Date(iso));
}

export function fmtTemp(c: number): string {
  return `${new Intl.NumberFormat(getLocale(), { maximumFractionDigits: 1 }).format(c)} °C`;
}

export function fmtHumidity(pct: number): string {
  return `${new Intl.NumberFormat(getLocale(), { maximumFractionDigits: 0 }).format(pct)} %`;
}
