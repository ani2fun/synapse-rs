/**
 * The one observable primitive the stateful islands share (A06; A07/A08 reuse it).
 *
 * Deliberately not a signals library: the four Preact islands need exactly "hold a value,
 * update it, re-render subscribers", and `useSyncExternalStore` is the platform-blessed way to
 * couple that to Preact without adding a reactivity runtime. The old client's RwSignal usage
 * maps 1:1 — `get_untracked` → `get()`, `set`/`update` likewise — which keeps the ports legible
 * against their Rust originals.
 */
import { useSyncExternalStore } from "preact/compat";

export class Store<T> {
  private value: T;
  private listeners = new Set<() => void>();

  constructor(initial: T) {
    this.value = initial;
  }

  get(): T {
    return this.value;
  }

  set(next: T): void {
    if (Object.is(next, this.value)) return;
    this.value = next;
    for (const listener of this.listeners) listener();
  }

  update(mutate: (current: T) => T): void {
    this.set(mutate(this.value));
  }

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }
}

/** Subscribe a Preact component to a store — re-renders on every `set`. */
export function useStore<T>(store: Store<T>): T {
  return useSyncExternalStore(
    (onChange) => store.subscribe(onChange),
    () => store.get(),
  );
}
