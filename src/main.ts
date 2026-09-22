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
    const modsBtn = document.createElement("button");
    modsBtn.textContent = "Mods";
    const modList = document.createElement("div");
    modsBtn.onclick = async () => {
      const files = await invoke<string[]>("list_instance_mods", { instance: i.name });
      const locked = await invoke<{ project_id: string; version_number: string; file_name: string }[]>("locked_mods", { instance: i.name });
      const updates: Record<string, { latest: string; installed: string; projectId: string }> = {};
      for (const m of locked) {
        try {
          const vs = await invoke<ProjectVersion[]>("project_versions", {
            projectId: m.project_id,
            gameVersions: [i.game_version],
            loaders: [i.loader],
          });
          if (vs[0] && vs[0].version_number !== m.version_number) {
            updates[m.file_name] = { latest: vs[0].version_number, installed: m.version_number, projectId: m.project_id };
          }
        } catch {
          // offline or untracked: no badge
        }
      }
      modList.innerHTML = "";
      for (const f of files) {
        const row = document.createElement("div");
        const name = document.createElement("span");
        const upd = updates[f];
        name.textContent = upd ? `${f} — update: ${upd.installed} → ${upd.latest}` : f;
        row.append(name);
        let updateBtn: HTMLButtonElement | null = null;
        if (upd) {
          updateBtn = document.createElement("button");
          updateBtn.textContent = "Update";
          updateBtn.onclick = async () => {
            const btn = updateBtn!;
            btn.disabled = true;
            btn.textContent = "Updating…";
            try {
              const vs = await invoke<ProjectVersion[]>("project_versions", {
                projectId: upd.projectId,
                gameVersions: [i.game_version],
                loaders: [i.loader],
              });
              const v = vs[0];
              const file = v?.files.find((x) => x.primary) ?? v?.files[0];
              if (!v || !file) throw new Error("no compatible file");
              if (file.name !== f) {
                await invoke<string[]>("remove_mod", { instance: i.name, fileName: f });
              }
              await invoke("install_mod", {
                instance: i.name,
                fileName: file.name,
                url: file.url,
                sha512: file.sha512,
                projectId: upd.projectId,
                versionNumber: v.version_number,
              });
              name.textContent = file.name;
              btn.remove();
              updateBtn = null;
            } catch (e) {
              btn.disabled = false;
              btn.textContent = "Update";
              name.textContent = `${f} — update failed: ${e}`;
            }
          };
          row.append(updateBtn);
        }
        const rm = document.createElement("button");
        rm.textContent = "Remove";
        rm.onclick = async () => {
          const rest = await invoke<string[]>("remove_mod", { instance: i.name, fileName: f });
          row.remove();
          label.textContent = `${i.name} — ${i.game_version} (${i.loader}): ${rest.length} mods`;
        };
        row.append(rm);
        modList.appendChild(row);
      }
      label.textContent = `${i.name} — ${i.game_version} (${i.loader}): ${files.length} mods`;
    };
    div.append(label, modsBtn);
    instEl.appendChild(div);
    instEl.appendChild(modList);
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
    projectId,
    versionNumber: v.version_number,
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

invoke<Account[]>("list_accounts").then((a) => {
  renderAccounts(a);
  refreshPlaySelectors();
});
invoke<Instance[]>("list_instances").then((l) => {
  renderInstances(l);
  refreshPlaySelectors();
});

const playAccount = document.querySelector<HTMLSelectElement>("#play-account")!;
const playInstance = document.querySelector<HTMLSelectElement>("#play-instance")!;
const playBtn = document.querySelector<HTMLButtonElement>("#play-btn")!;
const playStatus = document.querySelector<HTMLDivElement>("#play-status")!;

async function refreshPlaySelectors() {
  const [accounts, instances] = await Promise.all([
    invoke<Account[]>("list_accounts"),
    invoke<Instance[]>("list_instances"),
  ]);
  playAccount.innerHTML = accounts.map((a) => `<option value="${a.id}">${a.mc_name} [${a.kind}]</option>`).join("");
  playInstance.innerHTML = instances.map((i) => `<option value="${i.name}">${i.name} — ${i.game_version} (${i.loader})</option>`).join("");
}

playBtn.onclick = async () => {
  const accountId = playAccount.value;
  const instanceName = playInstance.value;
  if (!accountId || !instanceName) {
    playStatus.textContent = "Pick an account and an instance first.";
    return;
  }
  const [accounts, instances] = await Promise.all([
    invoke<Account[]>("list_accounts"),
    invoke<Instance[]>("list_instances"),
  ]);
  const acc = accounts.find((a) => a.id === accountId)!;
  const inst = instances.find((i) => i.name === instanceName)!;
  playStatus.textContent = `Launching ${inst.name} (${inst.game_version}) as ${acc.mc_name}… (libraries + assets download first)`;
  try {
    const r = await invoke<{ pid: number }>("launch_game", {
      version: inst.game_version,
      accountId: acc.id,
      accountName: acc.mc_name,
      accountUuid: acc.uuid,
      accessToken: (acc.ms_refresh as { accessToken?: string } | undefined)?.accessToken,
      instance: inst.name,
      loader: inst.loader,
    });
    playStatus.textContent = `Game started (pid ${r.pid}).`;
  } catch (e) {
    playStatus.textContent = `Launch failed: ${e}`;
  }
};

const logBtn = document.querySelector<HTMLButtonElement>("#log-btn")!;
const gameLog = document.querySelector<HTMLPreElement>("#game-log")!;
logBtn.onclick = async () => {
  try {
    gameLog.textContent = await invoke<string>("read_log_tail", {
      instance: playInstance.value || null,
      lines: 40,
    });
  } catch (e) {
    gameLog.textContent = `No log: ${e}`;
  }
};
