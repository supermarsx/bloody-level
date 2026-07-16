// A tiny event bus the layout uses to relock the UI when any command returns
// a `locked` error. Lives outside Svelte's reactive system intentionally —
// listeners can be plain functions, set up at app boot.

export type AuthEvent = { type: "locked"; command: string | null };

type Listener = (e: AuthEvent) => void;

class AuthEventBus {
  private listeners = new Set<Listener>();

  on(fn: Listener): () => void {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }

  emit(e: AuthEvent) {
    for (const l of this.listeners) {
      try {
        l(e);
      } catch (err) {
        console.error("auth-event listener threw", err);
      }
    }
  }
}

export const authEvents = new AuthEventBus();
