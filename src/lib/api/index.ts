import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { AppError } from "./errors";
import { authEvents } from "./auth-events.svelte";

export * from "./errors";

const IN_FLIGHT_TIMEOUT_MS = 30_000;

export interface InvokeOptions {
  /** Override the in-flight timeout (ms). Set to 0 to disable. */
  timeoutMs?: number;
  /** Don't broadcast `locked` errors to the auth-event bus. */
  silentAuth?: boolean;
}

/**
 * Wrapped Tauri invoke. Differences from raw invoke():
 *   - Errors are normalized into typed `AppError` instances.
 *   - A timeout fires `cancelled` if the IPC channel hangs.
 *   - `locked` errors are broadcast on the auth-event bus so the layout can
 *     re-lock the UI (one place, not 12).
 */
export async function invoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
  opts: InvokeOptions = {},
): Promise<T> {
  const timeoutMs = opts.timeoutMs ?? IN_FLIGHT_TIMEOUT_MS;

  const run = async (): Promise<T> => {
    const work = tauriInvoke<T>(cmd, args);

    let timer: ReturnType<typeof setTimeout> | null = null;
    const timeout: Promise<never> =
      timeoutMs > 0
        ? new Promise((_, reject) => {
            timer = setTimeout(() => {
              reject(
                new AppError(
                  {
                    kind: "cancelled",
                    code: "ipc.timeout",
                    message: `${cmd} did not respond within ${(timeoutMs / 1000).toFixed(0)}s`,
                    detail: null,
                    retryable: true,
                    timestamp: Math.floor(Date.now() / 1000),
                  },
                  cmd,
                ),
              );
            }, timeoutMs);
          })
        : new Promise<never>(() => {});

    try {
      const result = await Promise.race([work, timeout]);
      if (timer) clearTimeout(timer);
      return result;
    } finally {
      if (timer) clearTimeout(timer);
    }
  };

  // The Tauri custom-protocol IPC sometimes fails the first call after page
  // load (and occasionally subsequent calls under load); Tauri logs "IPC
  // custom protocol failed, will use the postMessage interface instead" and
  // the JS fetch rejects with a TypeError. Retry up to 3 times with
  // exponential backoff so the postMessage fallback has time to engage.
  const RETRY_DELAYS_MS = [50, 200, 500];
  let lastErr: unknown = null;
  for (let attempt = 0; attempt <= RETRY_DELAYS_MS.length; attempt++) {
    try {
      return await run();
    } catch (raw) {
      lastErr = raw;
      if (attempt < RETRY_DELAYS_MS.length && isIpcProtocolFailure(raw)) {
        // eslint-disable-next-line no-console
        console.warn(
          `[ipc] ${cmd} attempt ${attempt + 1} failed, retrying in ${RETRY_DELAYS_MS[attempt]}ms`,
        );
        await new Promise((r) => setTimeout(r, RETRY_DELAYS_MS[attempt]));
        continue;
      }
      break;
    }
  }
  const err = AppError.fromUnknown(lastErr, cmd);
  if (err.kind === "locked" && !opts.silentAuth) {
    authEvents.emit({ type: "locked", command: cmd });
  }
  throw err;
}

function isIpcProtocolFailure(e: unknown): boolean {
  if (e instanceof TypeError) {
    const msg = (e.message || "").toLowerCase();
    return (
      msg.includes("failed to fetch") ||
      msg.includes("network") ||
      msg.includes("load failed")
    );
  }
  return false;
}

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export * as auth from "./auth";
export * as settings from "./settings";
export * as ingest from "./ingest";
export * as reports from "./reports";
export * as tiers from "./tiers";
