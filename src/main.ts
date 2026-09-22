import { invoke } from "@tauri-apps/api/core";
import { addMicrosoftAccount, addOfflineAccount, type Account } from "./auth";

type Project = { id: string; title: string; description: string; downloads: number };

const q = document.querySelector<HTMLInputElement>("#q")!;
const go = document.querySelector<HTMLButtonElement>("#go")!;
const results = document.querySelector<HTMLDivElement>("#results")!;
const accountsEl = document.querySelector<HTMLDivElement>("#accounts")!;
const codeEl = document.querySelector<HTMLDivElement>("#code")!;
const offlineName = document.querySelector<HTMLInputElement>("#offline-name")!;
const addOfflineBtn = document.querySelector<HTMLButtonElement>("#add-offline")!;
const addMsBtn = document.querySelector<HTMLButtonElement>("#add-ms")!;

function renderAccounts(accounts: Account[]) {
  accountsEl.innerHTML = "";
  for (const a of accounts) {
    const div = document.createElement("div");
    div.className = "card";
    const label = document.createElement("span");
    label.textContent = `${a.mc_name} [${a.kind}]`;
    const del = document.createElement("button");
    del.textContent = "Remove";
    del.onclick = async () => {
      renderAccounts(await invoke<Account[]>("remove_account", { id: a.id }));
    };
    div.append(label, del);
    accountsEl.appendChild(div);
  }
  if (accounts.length === 0) accountsEl.textContent = "No accounts yet.";
}

addOfflineBtn.onclick = async () => {
  const name = offlineName.value.trim();
  if (name.length === 0) return;
  renderAccounts(await addOfflineAccount(name));
};

addMsBtn.onclick = async () => {
  codeEl.textContent = "Waiting for Microsoft…";
  try {
    renderAccounts(
      await addMicrosoftAccount((code, uri) => {
        codeEl.textContent = `Enter code ${code} at ${uri}`;
      }),
    );
    codeEl.textContent = "";
  } catch (e) {
    codeEl.textContent = `Sign-in failed: ${e}`;
  }
};

go.onclick = async () => {
  results.textContent = "Searching…";
  try {
    const hits = await invoke<Project[]>("search_mods", { text: q.value, limit: 20 });
    results.innerHTML = "";
    for (const h of hits) {
      const div = document.createElement("div");
      div.className = "card";
      div.innerHTML = `<strong></strong><span></span>`;
      div.querySelector("strong")!.textContent = h.title;
      div.querySelector("span")!.textContent = ` — ${h.downloads} downloads`;
      results.appendChild(div);
    }
    if (hits.length === 0) results.textContent = "No results.";
  } catch (e) {
    results.textContent = `Error: ${e}`;
  }
};

invoke<Account[]>("list_accounts").then(renderAccounts);
