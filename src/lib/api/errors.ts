export type ErrorKind =
  | 'locked'
  | 'not_initialized'
  | 'already_initialized'
  | 'not_found'
  | 'conflict'
  | 'bad_request'
  | 'crypto'
  | 'database'
  | 'filesystem'
  | 'pdf'
  | 'parser'
  | 'tier'
  | 'network'
  | 'cancelled'
  | 'unsupported'
  | 'internal';

export interface ErrorContext {
  stage: string | null;
  path: string | null;
  line: number | null;
  section: string | null;
  patient: string | null;
  command: string | null;
  hints: string[];
  breadcrumbs: string[];
}

export interface AppErrorPayload {
  kind: ErrorKind;
  code: string;
  message: string;
  detail: string | null;
  retryable: boolean;
  timestamp: number;
  context?: ErrorContext | null;
}

export class AppError extends Error {
  readonly kind: ErrorKind;
  readonly code: string;
  readonly detail: string | null;
  readonly retryable: boolean;
  readonly command: string | null;
  readonly timestamp: number;
  readonly context: ErrorContext | null;

  constructor(payload: AppErrorPayload, command: string | null = null) {
    super(payload.message);
    this.name = 'AppError';
    this.kind = payload.kind;
    this.code = payload.code;
    this.detail = payload.detail;
    this.retryable = payload.retryable;
    this.command = command;
    this.timestamp = payload.timestamp;
    this.context = payload.context ?? null;
  }

  static fromUnknown(e: unknown, command?: string): AppError {
    if (e instanceof AppError) return e;

    if (typeof e === 'object' && e !== null) {
      const obj = e as Record<string, unknown>;
      // Backend payload shape
      if (typeof obj.kind === 'string' && typeof obj.message === 'string') {
        return new AppError(
          {
            kind: obj.kind as ErrorKind,
            code: typeof obj.code === 'string' ? obj.code : 'unknown',
            message: obj.message as string,
            detail: typeof obj.detail === 'string' ? obj.detail : null,
            retryable: !!obj.retryable,
            timestamp: typeof obj.timestamp === 'number' ? obj.timestamp : Math.floor(Date.now() / 1000),
            context: (obj.context as ErrorContext | null) ?? null
          },
          command ?? null
        );
      }
    }

    if (e instanceof Error) {
      return new AppError(
        {
          kind: 'internal',
          code: 'frontend.exception',
          message: e.message || 'Unexpected error',
          detail: e.stack ?? null,
          retryable: false,
          timestamp: Math.floor(Date.now() / 1000)
        },
        command ?? null
      );
    }

    const message = typeof e === 'string' ? e : JSON.stringify(e);
    return new AppError(
      {
        kind: 'internal',
        code: 'unknown',
        message: message || 'Unknown error',
        detail: null,
        retryable: false,
        timestamp: Math.floor(Date.now() / 1000)
      },
      command ?? null
    );
  }

  isAuth(): boolean {
    return this.kind === 'locked' || this.kind === 'not_initialized';
  }

  isUserCorrectable(): boolean {
    return (
      this.kind === 'bad_request' ||
      this.kind === 'crypto' ||
      this.kind === 'conflict' ||
      this.kind === 'unsupported'
    );
  }
}

export const ERROR_TITLES: Record<ErrorKind, string> = {
  locked:               'Locked',
  not_initialized:      'Not initialized',
  already_initialized:  'Already initialized',
  not_found:            'Not found',
  conflict:             'Conflict',
  bad_request:          'Invalid input',
  crypto:               'Decryption failed',
  database:             'Database error',
  filesystem:           'Filesystem error',
  pdf:                  'PDF error',
  parser:               'Parse error',
  tier:                 'Tier unavailable',
  network:              'Network error',
  cancelled:            'Cancelled',
  unsupported:          'Not supported',
  internal:             'Unexpected error'
};

export function titleForError(e: AppError): string {
  return ERROR_TITLES[e.kind] ?? 'Error';
}
