export type TauriEventListen<T> = (
  event: string,
  handler: (event: { payload: T }) => void,
) => Promise<() => void>;

/**
 * The subscription only exists once `listen` resolves, so an unmount that races the
 * setup has to unlisten through the promise instead of a captured value.
 */
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
