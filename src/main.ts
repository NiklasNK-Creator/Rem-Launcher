import { invoke } from "@tauri-apps/api/core";
import { addMicrosoftAccount, addOfflineAccount, type Account } from "./auth";

type Project = { id: string; title: string; description: string; downloads: number };
type Instance = { name: string; game_version: string; loader: string };
type VersionFile = { name: string; url: string; sha512: string | null; primary: boolean };
type ProjectVersion = { version_number: string; files: VersionFile[] };

const q = document.querySelector<HTMLInputElement>("#q")!;
const go = document.querySelector<HTMLButtonElement>("#go")!;
const results = document.querySelector<HTMLDivElement>("#results")!;
const accountsEl = document.querySelector<HTMLDivElement>("#accounts")!;
const codeEl = document.querySelector<HTMLDivElement>("#code")!;
const offlineName = document.querySelector<HTMLInputElement>("#offline-name")!;
const addOfflineBtn = document.querySelector<HTMLButtonElement>("#add-offline")!;
const addMsBtn = document.querySelector<HTMLButtonElement>("#add-ms")!;
const instEl = document.querySelector<HTMLDivElement>("#instances")!;
const instName = document.querySelector<HTMLInputElement>("#inst-name")!;
const instVersion = document.querySelector<HTMLInputElement>("#inst-version")!;
const instLoader = document.querySelector<HTMLInputElement>("#inst-loader")!;
const createInst = document.querySelector<HTMLButtonElement>("#create-inst")!;

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

async function installToFirstInstance(projectId: string) {
  const list = await invoke<Instance[]>("list_instances");
  const target = list[0];
  if (!target) {
    results.textContent = "Create an instance first.";
    return;
  }
  const versions = await invoke<ProjectVersion[]>("project_versions", {
    projectId,
    gameVersions: [target.game_version],
    loaders: [target.loader],
  });
  const v = versions[0];
  const f = v?.files.find((x) => x.primary) ?? v?.files[0];
  if (!v || !f) {
    results.textContent = "No compatible version for this instance.";
    return;
  }
  await invoke("install_mod", {
    instance: target.name,
    fileName: f.name,
    url: f.url,
    sha512: f.sha512,
  });
  results.textContent = `Installed ${f.name} (${v.version_number}) into ${target.name}.`;
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
      const title = document.createElement("strong");
      title.textContent = h.title;
      const meta = document.createElement("span");
      meta.textContent = ` — ${h.downloads} downloads`;
      const btn = document.createElement("button");
      btn.textContent = "Install";
      btn.onclick = () => installToFirstInstance(h.id);
      div.append(title, meta, btn);
      results.appendChild(div);
    }
    if (hits.length === 0) results.textContent = "No results.";
  } catch (e) {
    results.textContent = `Error: ${e}`;
  }
};

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
