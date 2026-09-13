type EnhanceResult = { type: string; data?: { error?: string; data?: unknown } };
type Callback = (args: { update: () => Promise<void>; result: EnhanceResult }) => Promise<void>;

export function actionState() {
  let error = $state<string | null>(null);

  return {
    get error() {
      return error;
    },
    track:
      (onSuccess?: (data: unknown) => void) =>
      (): Callback =>
      async ({ update, result }) => {
        await update();
        error = result.type === 'failure' ? (result.data?.error ?? 'unknown') : null;
        if (result.type === 'success') onSuccess?.(result.data?.data);
      },
    open(fn: () => void) {
      error = null;
      fn();
    },
    close(fn: () => void) {
      error = null;
      fn();
    }
  };
}
