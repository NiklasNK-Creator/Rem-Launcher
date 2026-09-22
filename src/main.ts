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

type Instance = { name: string; game_version: string; loader: string };

const instEl = document.querySelector<HTMLDivElement>("#instances")!;
const instName = document.querySelector<HTMLInputElement>("#inst-name")!;
const instVersion = document.querySelector<HTMLInputElement>("#inst-version")!;
const instLoader = document.querySelector<HTMLInputElement>("#inst-loader")!;
const createInst = document.querySelector<HTMLButtonElement>("#create-inst")!;

function renderInstances(list: Instance[]) {
  instEl.innerHTML = "";
  for (const i of list) {
    const div = document.createElement("div");
    div.className = "card";
    const label = document.createElement("span");
    label.textContent = `${i.name} — ${i.game_version} (${i.loader})`;
    const mods = document.createElement("button");
    mods.textContent = "Mods";
    mods.onclick = async () => {
      const files = await invoke<string[]>("list_instance_mods", { instance: i.name });
      label.textContent = `${i.name} — ${i.game_version} (${i.loader}): ${files.length} mods`;
    };
    div.append(label, mods);
    instEl.appendChild(div);
  }
  if (list.length === 0) instEl.textContent = "No instances yet.";
}

createInst.onclick = async () => {
  try {
    renderInstances(
      await invoke<Instance[]>("create_instance", {
        name: instName.value.trim(),
        gameVersion: instVersion.value.trim(),
        loader: instLoader.value.trim() || "fabric",
      }),
    );
  } catch (e) {
    instEl.textContent = `Error: ${e}`;
  }
};

invoke<Account[]>("list_accounts").then(renderAccounts);
invoke<Instance[]>("list_instances").then(renderInstances);
