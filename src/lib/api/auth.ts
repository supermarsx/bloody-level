import { invoke } from "./index";

export interface AuthStatus {
  initialized: boolean;
  has_password: boolean;
  has_passkey: boolean;
  passkey_count: number;
  passkeys: PasskeySummary[];
  unlocked: boolean;
  failed_unlocks: number;
  unlock_backoff_remaining_secs: number;
  is_dev: boolean;
}

export interface PasskeySummary {
  label: string;
  credential_id_b64: string;
  prf_salt_b64: string;
}

export async function status(): Promise<AuthStatus> {
  // silentAuth: status() itself must never trigger a relock cascade.
  return invoke<AuthStatus>("auth_status", undefined, { silentAuth: true });
}

export async function setupPassword(password: string): Promise<void> {
  await invoke("auth_setup_password", { password });
}

export async function unlockPassword(password: string): Promise<void> {
  await invoke("auth_unlock_password", { password }, { silentAuth: true });
}

export async function changePassword(
  currentPassword: string,
  newPassword: string,
): Promise<void> {
  await invoke("auth_change_password", {
    args: { current_password: currentPassword, new_password: newPassword },
  });
}

export async function lock(): Promise<void> {
  await invoke("auth_lock", undefined, { silentAuth: true });
}

export async function registerPasskey(args: {
  label: string;
  credential_id_b64: string;
  prf_salt_b64: string;
  prf_output_b64: string;
  current_password: string;
}): Promise<void> {
  await invoke("auth_register_passkey", { args });
}

export async function unlockPasskey(args: {
  credential_id_b64: string;
  prf_output_b64: string;
}): Promise<void> {
  await invoke("auth_unlock_passkey", { args }, { silentAuth: true });
}

export const DEV_SKIP_PASSWORD = "dev-skip-not-for-production-x7q2";

export async function devSkip(): Promise<void> {
  const s = await status();
  if (!s.is_dev) {
    throw new Error("dev skip is not available in release builds");
  }
  if (s.initialized) {
    await unlockPassword(DEV_SKIP_PASSWORD);
  } else {
    await setupPassword(DEV_SKIP_PASSWORD);
  }
}

// ---------------------------------------------------------------------------
// WebAuthn / PRF helpers (frontend-only)
// ---------------------------------------------------------------------------

const RP_ID = "localhost";
const RP_NAME = "bloody-level";

function bytesToB64(bytes: ArrayBuffer | Uint8Array): string {
  const u = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  let s = "";
  for (let i = 0; i < u.length; i++) s += String.fromCharCode(u[i]);
  return btoa(s);
}

function b64ToBytes(b64: string): Uint8Array<ArrayBuffer> {
  const bin = atob(b64);
  const ab = new ArrayBuffer(bin.length);
  const u = new Uint8Array(ab);
  for (let i = 0; i < bin.length; i++) u[i] = bin.charCodeAt(i);
  return u;
}

export function isWebAuthnAvailable(): boolean {
  return (
    typeof navigator !== "undefined" &&
    !!navigator.credentials &&
    typeof window !== "undefined" &&
    !!window.PublicKeyCredential
  );
}

export async function isPrfAvailable(): Promise<boolean> {
  return isWebAuthnAvailable();
}

export interface PasskeyRegistration {
  credentialId: Uint8Array;
  prfSalt: Uint8Array;
  prfOutput: Uint8Array | null;
}

export async function webauthnRegister(
  userName: string,
): Promise<PasskeyRegistration> {
  const challenge = crypto.getRandomValues(new Uint8Array(32));
  const userId = crypto.getRandomValues(new Uint8Array(16));
  const prfSalt = crypto.getRandomValues(new Uint8Array(32));

  const cred = (await navigator.credentials.create({
    publicKey: {
      rp: { id: RP_ID, name: RP_NAME },
      user: { id: userId, name: userName, displayName: userName },
      challenge,
      pubKeyCredParams: [
        { type: "public-key", alg: -7 },
        { type: "public-key", alg: -257 },
      ],
      authenticatorSelection: {
        residentKey: "required",
        userVerification: "required",
      },
      timeout: 60000,
      extensions: {
        prf: { eval: { first: prfSalt } },
      } as AuthenticationExtensionsClientInputs,
    },
  })) as PublicKeyCredential | null;

  if (!cred) throw new Error("credential creation cancelled");

  const credId = new Uint8Array(cred.rawId);
  const ext =
    cred.getClientExtensionResults() as AuthenticationExtensionsClientOutputs & {
      prf?: { results?: { first?: ArrayBuffer } };
    };
  const prfOutput = ext.prf?.results?.first
    ? new Uint8Array(ext.prf.results.first)
    : null;

  return { credentialId: credId, prfSalt, prfOutput };
}

export interface PasskeyAssertion {
  credentialId: Uint8Array;
  prfOutput: Uint8Array;
}

export async function webauthnAssert(
  credentialIdB64: string,
  prfSaltB64: string,
): Promise<PasskeyAssertion> {
  const challenge = crypto.getRandomValues(new Uint8Array(32));
  const allow = [
    { type: "public-key" as const, id: b64ToBytes(credentialIdB64) },
  ];
  const prfSalt = b64ToBytes(prfSaltB64);

  const assertion = (await navigator.credentials.get({
    publicKey: {
      challenge,
      rpId: RP_ID,
      allowCredentials: allow,
      userVerification: "required",
      timeout: 60000,
      extensions: {
        prf: { eval: { first: prfSalt } },
      } as AuthenticationExtensionsClientInputs,
    },
  })) as PublicKeyCredential | null;

  if (!assertion) throw new Error("assertion cancelled");

  const credId = new Uint8Array(assertion.rawId);
  const ext =
    assertion.getClientExtensionResults() as AuthenticationExtensionsClientOutputs & {
      prf?: { results?: { first?: ArrayBuffer } };
    };
  if (!ext.prf?.results?.first) {
    throw new Error("PRF not available — use password fallback");
  }
  return {
    credentialId: credId,
    prfOutput: new Uint8Array(ext.prf.results.first),
  };
}

export const b64 = { encode: bytesToB64, decode: b64ToBytes };
