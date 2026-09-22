import { invoke } from "@tauri-apps/api/core";
import { Authflow } from "prismarine-auth";

export type Account = {
  id: string;
  kind: "microsoft" | "offline";
  mc_name: string;
  uuid: string;
  ms_refresh?: unknown;
  last_used?: string | null;
};

// Deterministic offline-mode UUID for a name (vanilla uses MD5 of
// "OfflinePlayer:"+name; offline servers ignore the uuid, so a stable
// FNV-derived v3-style value is enough to stay consistent per name).
function offlineUuid(name: string): string {
  const s = "OfflinePlayer:" + name;
  let a = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) {
    a ^= s.charCodeAt(i);
    a = Math.imul(a, 0x01000193) >>> 0;
  }
  const hex = a.toString(16).padStart(8, "0");
  const h = (hex + hex + hex + hex).slice(0, 32);
  return `${h.slice(0, 8)}-${h.slice(8, 12)}-3${h.slice(13, 16)}-a${h.slice(17, 20)}-${h.slice(20)}`.replaceAll("-", "");
}

export async function addOfflineAccount(name: string): Promise<Account[]> {
  return invoke("add_account", {
    account: {
      id: `offline:${name}`,
      kind: "offline",
      mc_name: name,
      uuid: offlineUuid(name),
    } satisfies Account,
  });
}

export async function addMicrosoftAccount(
  onCode: (code: string, uri: string) => void,
): Promise<Account[]> {
  // NOTE: token cache is in-memory (prismarine-auth default); the captured
  // access token expires (~24h). Re-adding the account refreshes it.
  const flow = new Authflow(`ms:${Date.now()}`, undefined, {}, (code: { user_code: string; verification_uri: string }) =>
    onCode(code.user_code, code.verification_uri),
  );
  const { token, profile } = await flow.getMinecraftJavaToken({ fetchProfile: true });
  const p = profile as { id: string; name: string };
  return invoke("add_account", {
    account: {
      id: `microsoft:${p.id}`,
      kind: "microsoft",
      mc_name: p.name,
      uuid: p.id,
      ms_refresh: { accessToken: token },
    } satisfies Account,
  });
}
