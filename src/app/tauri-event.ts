export type TauriEventListen<T> = (
  event: string,
  handler: (event: { payload: T }) => void,
) => Promise<() => void>;

/** The subscription exists only once `listen` resolves, so cleanup must unlisten through the promise. */
export function subscribeToTauriEvent<T>(
  listenFn: TauriEventListen<T>,
  event: string,
  onPayload: (payload: T) => void,
): () => void {
  const subscription = listenFn(event, ({ payload }) => onPayload(payload));
  return () => {
    void subscription.then((unlisten) => unlisten()).catch(() => {});
  };
}
