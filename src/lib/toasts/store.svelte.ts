import { AppError, titleForError, type ErrorContext } from "$api/errors";

export type ToastKind = "info" | "success" | "warn" | "error";

export interface Toast {
  id: number;
  kind: ToastKind;
  title: string;
  message: string | null;
  detail?: string | null;
  retry?: () => void;
  durationMs: number;
  command?: string | null;
  code?: string | null;
  context?: ErrorContext | null;
}

class ToastStore {
  toasts = $state<Toast[]>([]);
  private nextId = 1;

  private push(t: Omit<Toast, "id">): number {
    const id = this.nextId++;
    const toast: Toast = { ...t, id };
    this.toasts = [...this.toasts, toast];
    if (toast.durationMs > 0) {
      setTimeout(() => this.dismiss(id), toast.durationMs);
    }
    return id;
  }

  info(title: string, message?: string, durationMs = 4000): number {
    return this.push({
      kind: "info",
      title,
      message: message ?? null,
      durationMs,
    });
  }

  success(title: string, message?: string, durationMs = 3000): number {
    return this.push({
      kind: "success",
      title,
      message: message ?? null,
      durationMs,
    });
  }

  warn(title: string, message?: string, durationMs = 5000): number {
    return this.push({
      kind: "warn",
      title,
      message: message ?? null,
      durationMs,
    });
  }

  /** Push a typed error. Sticky for crypto/locked; auto-dismiss otherwise. */
  error(
    err: unknown,
    opts: { retry?: () => void; sticky?: boolean } = {},
  ): number {
    const e = err instanceof AppError ? err : AppError.fromUnknown(err);
    const sticky =
      opts.sticky ??
      (e.kind === "crypto" || e.kind === "locked" || e.kind === "database");
    return this.push({
      kind: "error",
      title: titleForError(e),
      message: e.message,
      detail: e.detail,
      retry: opts.retry,
      durationMs: sticky ? 0 : 7000,
      command: e.command,
      code: e.code,
      context: e.context,
    });
  }

  dismiss(id: number) {
    this.toasts = this.toasts.filter((t) => t.id !== id);
  }

  clear() {
    this.toasts = [];
  }
}

export const toasts = new ToastStore();
