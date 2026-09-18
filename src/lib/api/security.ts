import { invoke } from "./index";

export interface SecurityStatus {
  os_vault_supported: boolean;
  os_vault_platform: string;
  os_vault_enabled: boolean;
  os_vault_auto_unlock: boolean;
  os_vault_credential_present: boolean;
  session_unlocked: boolean;
  last_error: string | null;
}

export function status(): Promise<SecurityStatus> {
  return invoke("security_status");
}

export function enableOsVault(): Promise<SecurityStatus> {
  return invoke("security_enable_os_vault");
}

export function disableOsVault(): Promise<SecurityStatus> {
  return invoke("security_disable_os_vault", { confirm: true });
}

export function setAutoUnlock(enabled: boolean): Promise<SecurityStatus> {
  return invoke("security_set_auto_unlock", { enabled });
}

export function unlockOsVault(): Promise<SecurityStatus> {
  return invoke("security_unlock_os_vault", undefined, { silentAuth: true });
}
