import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { addMicrosoftAccount, addOfflineAccount, refreshMicrosoftAccount, type Account } from "./auth";

type Project = { id: string; title: string; description: string; downloads: number; follows?: number | null; updated?: string | null; icon_url?: string | null };
type Instance = { name: string; game_version: string; loader: string; max_memory_mb?: number | null; jvm_args?: string | null; java_path?: string | null };
type VersionFile = { name: string; url: string; sha512: string | null; primary: boolean };
type VersionDep = { project_id: string | null; dependency_type: string };
type ProjectVersion = { version_number: string; files: VersionFile[]; dependencies: VersionDep[] };

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
const instVersion = document.querySelector<HTMLSelectElement>("#inst-version")!;
invoke<string[]>("list_versions", { kind: "release" }).then((vs) => {
  instVersion.innerHTML = vs.slice(0, 30).map((v) => `<option value="${v}">${v}</option>`).join("");
  const sv = document.querySelector<HTMLSelectElement>("#search-version")!;
  sv.innerHTML = `<option value="">Any version</option>` + vs.slice(0, 30).map((v) => `<option value="${v}">${v}</option>`).join("");
});
const instLoader = document.querySelector<HTMLSelectElement>("#inst-loader")!;
const createInst = document.querySelector<HTMLButtonElement>("#create-inst")!;

function renderAccounts(accounts: Account[]) {
  const sorted = [...accounts].sort((a, b) => (b.last_used ?? "").localeCompare(a.last_used ?? ""));
  accountsEl.innerHTML = "";
  for (const a of sorted) {
    const div = document.createElement("div");
    div.className = "card";
    if (a.skin_url) {
      const img = document.createElement("img");
      img.src = a.skin_url;
      img.width = 32;
      img.height = 32;
      img.alt = "";
      div.appendChild(img);
    }
    const label = document.createElement("span");
    const last = a.last_used ? ` · last used ${new Date(Number(a.last_used) * 1000).toLocaleDateString()}` : "";
    const cape = a.cape_id ? " · cape" : "";
    label.textContent = `${a.mc_name} [${a.kind}]${last}${cape}`;
    const useBtn = document.createElement("button");
    useBtn.textContent = "Use";
    useBtn.onclick = async () => {
      await invoke("touch_account", { id: a.id });
      const all = await invoke<Account[]>("list_accounts");
      renderAccounts(all);
      refreshPlaySelectors(a.id);
    };
    const del = document.createElement("button");
    del.textContent = "Remove";
    del.onclick = async () => {
      renderAccounts(await invoke<Account[]>("remove_account", { id: a.id }));
    };
    div.append(label, useBtn, del);
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
    const javaTag = i.java_path ? ` · Java: ${i.java_path}` : "";
    label.textContent = `${i.name} — ${i.game_version} (${i.loader}) [${i.max_memory_mb ?? 4096}MB${javaTag}]`;
    const modsBtn = document.createElement("button");
    modsBtn.textContent = "Mods";
    const verifyBtn = document.createElement("button");
    verifyBtn.textContent = "Verify";
    verifyBtn.onclick = async () => {
      const r = await invoke<{ missing_files: string[]; untracked_files: string[]; ok_tracked: number }>("verify_instance", { instance: i.name });
      label.textContent = `${i.name}: ${r.ok_tracked} ok, ${r.missing_files.length} missing, ${r.untracked_files.length} untracked`;
    };
    const repairBtn = document.createElement("button");
    repairBtn.textContent = "Repair";
    repairBtn.onclick = async () => {
      const r = await invoke<{ missing_files: string[]; untracked_files: string[]; ok_tracked: number }>("repair_instance", { instance: i.name });
      label.textContent = `${i.name}: repaired — ${r.ok_tracked} ok, ${r.untracked_files.length} untracked`;
    };
    const configureBtn = document.createElement("button");
    configureBtn.textContent = "Configure";
    configureBtn.onclick = async () => {
      const raw = prompt("Max memory in MB (blank = 4096):", String(i.max_memory_mb ?? 4096));
      if (raw === null) return;
      const memory = raw.trim() ? Number(raw) : 4096;
      if (!Number.isInteger(memory) || memory < 512 || memory > 65536) {
        label.textContent = "Memory must be an integer from 512 to 65536 MB";
        return;
      }
      const java = prompt("Java executable path (blank = system java):", i.java_path ?? "");
      if (java === null) return;
      const javaPath = java.trim() || null;
      try {
        await invoke("validate_java_path", { path: javaPath });
      } catch (e) {
        label.textContent = `Java path invalid: ${e}`;
        return;
      }
      const updated = await invoke<Instance[]>("configure_instance", {
        name: i.name,
        maxMemoryMb: memory,
        jvmArgs: i.jvm_args ?? null,
        javaPath,
      });
      renderInstances(updated);
    };
    const folderBtn = document.createElement("button");
    folderBtn.textContent = "Folder";
    folderBtn.onclick = async () => {
      try { await invoke("open_instance_folder", { name: i.name }); }
      catch (e) { label.textContent = `Folder failed: ${e}`; }
    };
    const cloneBtn = document.createElement("button");
    cloneBtn.textContent = "Clone";
    cloneBtn.onclick = async () => {
      const newName = prompt(`Clone ${i.name} as:`, `${i.name}-copy`);
      if (!newName || !newName.trim()) return;
      try {
        const updated = await invoke<Instance[]>("clone_instance", { sourceName: i.name, newName: newName.trim() });
        renderInstances(updated);
        refreshPlaySelectors();
      } catch (e) {
        label.textContent = `Clone failed: ${e}`;
      }
    };
    const delInst = document.createElement("button");
    delInst.textContent = "Delete";
    delInst.onclick = async () => {
      if (!confirm(`Delete instance ${i.name} and all its files?`)) return;
      renderInstances(await invoke<Instance[]>("delete_instance", { name: i.name }));
      refreshPlaySelectors();
    };
    const backupBtn = document.createElement("button");
    backupBtn.textContent = "Backup";
    backupBtn.onclick = async () => {
      try {
        const b = await invoke<{ file_name: string; size_bytes: number }>("create_backup", { instance: i.name, backupName: null });
        label.textContent = `${i.name}: backup ${b.file_name} (${b.size_bytes} bytes)`;
      } catch (e) { label.textContent = `Backup failed: ${e}`; }
    };
    const restoreBtn = document.createElement("button");
    restoreBtn.textContent = "Restore";
    restoreBtn.onclick = async () => {
      try {
        const backups = await invoke<{ file_name: string; size_bytes: number; created_at: string }[]>("list_backups", { instance: i.name });
        if (backups.length === 0) throw new Error("no backups");
        const chosen = prompt(`Restore which backup? Current saves will be overwritten.\nAvailable:\n${backups.map((b) => b.file_name).join("\n")}`, backups[0].file_name);
        if (!chosen) return;
        if (!confirm(`Restore ${chosen}? This overwrites the current saves of ${i.name}.`)) return;
        await invoke("restore_backup", { instance: i.name, fileName: chosen });
        label.textContent = `${i.name}: restored ${chosen}`;
      } catch (e) { label.textContent = `Restore failed: ${e}`; }
    };
    const manageBackupsBtn = document.createElement("button");
    manageBackupsBtn.textContent = "Backups";
    manageBackupsBtn.onclick = async () => {
      try {
        const backups = await invoke<{ file_name: string; size_bytes: number; created_at: string }[]>("list_backups", { instance: i.name });
        if (backups.length === 0) { label.textContent = `${i.name}: no backups yet`; return; }
        const picked = prompt(`Backups of ${i.name} (size bytes shown). Type a filename to delete it, or leave blank to keep all:\n${backups.map((b) => `${b.file_name} (${b.size_bytes} bytes)`).join("\n")}`, "");
        if (!picked || !picked.trim()) return;
        if (!confirm(`Delete backup ${picked.trim()}?`)) return;
        await invoke("delete_backup", { instance: i.name, fileName: picked.trim() });
        label.textContent = `${i.name}: deleted backup ${picked.trim()}`;
      } catch (e) { label.textContent = `Backup management failed: ${e}`; }
    };
    div.append(label, modsBtn, verifyBtn, repairBtn, configureBtn, folderBtn, cloneBtn, backupBtn, restoreBtn, manageBackupsBtn, delInst);
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
        const isDisabled = f.endsWith(".disabled");
        const cleanName = isDisabled ? f.slice(0, -9) : f;
        const upd = updates[f] ?? updates[cleanName];
        const displayName = isDisabled ? `${cleanName} [Disabled]` : f;
        name.textContent = upd ? `${displayName} — update: ${upd.installed} → ${upd.latest}` : displayName;
        if (isDisabled) {
          name.style.opacity = "0.6";
        }
        row.append(name);
        let updateBtn: HTMLButtonElement | null = null;
        if (upd && !isDisabled) {
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
        const toggleBtn = document.createElement("button");
        toggleBtn.textContent = isDisabled ? "Enable" : "Disable";
        toggleBtn.onclick = async () => {
          try {
            await invoke<string[]>("toggle_content_item", { instance: i.name, relativePath: f });
            modsBtn.click();
          } catch (e) {
            name.textContent = `${f} — toggle failed: ${e}`;
          }
        };
        const rm = document.createElement("button");
        rm.textContent = "Remove";
        rm.onclick = async () => {
          const rest = await invoke<string[]>("remove_content_item", { instance: i.name, relativePath: f });
          row.remove();
          label.textContent = `${i.name} — ${i.game_version} (${i.loader}): ${rest.length} items`;
        };
        row.append(toggleBtn, rm);
        modList.appendChild(row);
      }
      label.textContent = `${i.name} — ${i.game_version} (${i.loader}): ${files.length} items`;
    };
    instEl.appendChild(div);
    instEl.appendChild(modList);
  }
  if (list.length === 0) instEl.textContent = "No instances yet.";
}

async function installToFirstInstance(projectId: string, ct: string) {
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
  // Resolve full dep tree for preview (cycle-safe).
  type DepNode = { pid: string; title: string; version: string; kind: string; children: DepNode[] };
  const seen = new Set<string>([projectId]);
  async function resolve(pid: string, ver: ProjectVersion): Promise<DepNode[]> {
    const out: DepNode[] = [];
    for (const dep of ver.dependencies ?? []) {
      if (!dep.project_id || seen.has(dep.project_id)) continue;
      seen.add(dep.project_id);
      try {
        const dvs = await invoke<ProjectVersion[]>("project_versions", {
          projectId: dep.project_id, gameVersions: [target.game_version], loaders: [target.loader],
        });
        const dv = dvs[0];
        if (!dv) continue;
        const proj = await invoke<{ title: string }>("project_info", { projectId: dep.project_id }).catch(() => ({ title: dep.project_id }));
        out.push({ pid: dep.project_id, title: proj.title, version: dv.version_number, kind: dep.dependency_type, children: await resolve(dep.project_id, dv) });
      } catch {
        // unresolvable: skip
      }
    }
    return out;
  }
  const tree = await resolve(projectId, v);
  // Preview panel with optional-dep checkboxes.
  results.innerHTML = "";
  const panel = document.createElement("div");
  panel.className = "card";
  const heading = document.createElement("strong");
  heading.textContent = `Install ${f.name} (${v.version_number}) into ${target.name}?`;
  panel.appendChild(heading);
  const listEl = document.createElement("div");
  const optionalChecks: { pid: string; checked: () => boolean }[] = [];
  function renderNode(n: DepNode, depth: number) {
    const row = document.createElement("div");
    const label = `${"  ".repeat(depth)}${n.kind}: ${n.title} (${n.version})`;
    if (n.kind === "optional") {
      const box = document.createElement("input");
      box.type = "checkbox";
      box.checked = false;
      const span = document.createElement("span");
      span.textContent = label;
      row.append(box, span);
      optionalChecks.push({ pid: n.pid, checked: () => box.checked });
    } else {
      const span = document.createElement("span");
      span.textContent = label;
      row.appendChild(span);
    }
    listEl.appendChild(row);
    for (const c of n.children) renderNode(c, depth + 1);
  }
  for (const n of tree) renderNode(n, 1);
  panel.appendChild(listEl);
  const confirm = document.createElement("button");
  confirm.textContent = "Confirm install";
  const cancel = document.createElement("button");
  cancel.textContent = "Cancel";
  panel.append(confirm, cancel);
  results.appendChild(panel);
  const doInstall = async (pid: string, ver: ProjectVersion, wantOptional: Set<string>) => {
    for (const dep of ver.dependencies ?? []) {
      if (!dep.project_id) continue;
      const required = dep.dependency_type === "required";
      if (!required && !wantOptional.has(dep.project_id)) continue;
      try {
        const dvs = await invoke<ProjectVersion[]>("project_versions", {
          projectId: dep.project_id, gameVersions: [target.game_version], loaders: [target.loader],
        });
        const dv = dvs[0];
        const df = dv?.files.find((x) => x.primary) ?? dv?.files[0];
        if (!dv || !df) continue;
        await invoke("install_mod", {
          instance: target.name, fileName: df.name, url: df.url, sha512: df.sha512,
          projectId: dep.project_id, versionNumber: dv.version_number, contentType: "mod",
        });
        await doInstall(dep.project_id, dv, wantOptional);
      } catch {
        // skip uninstallable dep
      }
    }
    const file = ver.files.find((x) => x.primary) ?? ver.files[0];
    if (!file) return;
    await invoke("install_mod", {
      instance: target.name, fileName: file.name, url: file.url, sha512: file.sha512,
      projectId: pid, versionNumber: ver.version_number, contentType: ct,
    });
  };
  confirm.onclick = async () => {
    confirm.disabled = true;
    const want = new Set(optionalChecks.filter((o) => o.checked()).map((o) => o.pid));
    try {
      await doInstall(projectId, v, want);
      results.textContent = `Installed ${f.name} (${v.version_number}) into ${target.name}.`;
    } catch (e) {
      results.textContent = `Install failed: ${e}`;
    }
  };
  cancel.onclick = () => {
    results.textContent = "Install cancelled.";
  };
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

async function installPack(projectId: string) {
  results.textContent = "Resolving modpack…";
  try {
    const vs = await invoke<ProjectVersion[]>("project_versions", { projectId, gameVersions: [], loaders: [] });
    const v = vs[0];
    const file = v?.files.find((x) => x.primary) ?? v?.files[0];
    if (!v || !file || !file.name.endsWith(".mrpack")) throw new Error("no .mrpack file found");
    results.textContent = `Downloading ${file.name}…`;
    const r = await invoke<{ instance: string; files: number }>("install_mrpack_url", { url: file.url });
    const list = await invoke<Instance[]>("list_instances");
    renderInstances(list);
    refreshPlaySelectors();
    results.textContent = `Installed pack as ${r.instance} (${r.files} files).`;
  } catch (e) {
    results.textContent = `Pack install failed: ${e}`;
  }
}

go.onclick = async () => {
  results.textContent = "Searching…";
  const ct = (document.querySelector<HTMLSelectElement>("#content-type")!).value;
  const sv = (document.querySelector<HTMLSelectElement>("#search-version")!).value;
  const sl = (document.querySelector<HTMLSelectElement>("#search-loader")!).value;
  const sort = (document.querySelector<HTMLSelectElement>("#search-sort")!).value;
  try {
    const hits = await invoke<Project[]>("search_mods", {
      text: q.value, limit: 20, contentType: ct,
      gameVersion: sv || null, loader: sl || null, sort,
    });
    results.innerHTML = "";
    let page = 0;
    const render = (items: Project[]) => {
      for (const h of items) {
        const div = document.createElement("div");
        div.className = "card";
        if (h.icon_url) {
          const icon = document.createElement("img");
          icon.src = h.icon_url;
          icon.width = 40;
          icon.height = 40;
          icon.alt = "";
          icon.loading = "lazy";
          div.appendChild(icon);
        }
        const title = document.createElement("strong");
        title.textContent = h.title;
        const meta = document.createElement("span");
        const updated = h.updated ? ` · updated ${new Date(h.updated).toLocaleDateString()}` : "";
        meta.textContent = ` — ${h.downloads.toLocaleString()} downloads · ${(h.follows ?? 0).toLocaleString()} follows${updated}`;
        const btn = document.createElement("button");
        btn.textContent = ct === "modpack" ? "Install pack" : ct === "mod" ? "Install" : "Download";
        btn.onclick = () => (ct === "modpack" ? installPack(h.id) : installToFirstInstance(h.id, ct));
        div.append(title, meta, btn);
        results.appendChild(div);
      }
    };
    render(hits);
    if (hits.length === 0) results.textContent = "No results.";
    if (hits.length === 20) {
      const more = document.createElement("button");
      more.textContent = "Load more";
      more.onclick = async () => {
        page += 1;
        more.disabled = true;
        try {
          const next = await invoke<Project[]>("search_mods", {
            text: q.value, limit: 20, contentType: ct,
            gameVersion: sv || null, loader: sl || null, sort, offset: page * 20,
          });
          more.remove();
          render(next);
          if (next.length === 20) results.appendChild(more);
        } catch (e) {
          more.textContent = `Failed: ${e}`;
          more.disabled = false;
        }
      };
      results.appendChild(more);
    }
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

const mrpackPath = document.querySelector<HTMLInputElement>("#mrpack-path")!;
const importMrpack = document.querySelector<HTMLButtonElement>("#import-mrpack")!;
importMrpack.onclick = async () => {
  const p = mrpackPath.value.trim();
  if (!p) return;
  try {
    const r = await invoke<{ instance: string; files: number }>("import_mrpack", { path: p });
    const list = await invoke<Instance[]>("list_instances");
    renderInstances(list);
    refreshPlaySelectors();
    instEl.textContent = `Imported ${r.instance} (${r.files} files).`;
    renderInstances(list);
  } catch (e) {
    instEl.textContent = `Import failed: ${e}`;
  }
};

const exportPath = document.querySelector<HTMLInputElement>("#export-path")!;
const exportMrpack = document.querySelector<HTMLButtonElement>("#export-mrpack")!;
exportMrpack.onclick = async () => {
  const out = exportPath.value.trim();
  const inst = (document.querySelector<HTMLSelectElement>("#play-instance")!);
  if (!out || !inst.value) {
    instEl.textContent = "Pick an instance in Play and an export path first.";
    return;
  }
  try {
    const r = await invoke<{ path: string; files: number }>("export_mrpack", { instance: inst.value, outPath: out });
    instEl.textContent = `Exported ${r.files} entries to ${r.path}.`;
  } catch (e) {
    instEl.textContent = `Export failed: ${e}`;
  }
};
const playAccount = document.querySelector<HTMLSelectElement>("#play-account")!;
const playInstance = document.querySelector<HTMLSelectElement>("#play-instance")!;

async function refreshPlaySelectors(preferAccount?: string) {
  const [accounts, instances, servers] = await Promise.all([
    invoke<Account[]>("list_accounts"),
    invoke<Instance[]>("list_instances"),
    invoke<{ id: string; name: string; address: string; last_ping_ms?: number | null }[]>("list_servers"),
  ]);
  const sorted = [...accounts].sort((a, b) => (b.last_used ?? "").localeCompare(a.last_used ?? ""));
  playAccount.innerHTML = sorted.map((a) => `<option value="${a.id}">${a.mc_name} [${a.kind}]</option>`).join("");
  if (preferAccount && sorted.some((a) => a.id === preferAccount)) playAccount.value = preferAccount;
  else if (sorted[0]) playAccount.value = sorted[0].id;
  playInstance.innerHTML = instances.map((i) => `<option value="${i.name}">${i.name} — ${i.game_version} (${i.loader})</option>`).join("");
  const lastInst = localStorage.getItem("rem-last-instance");
  if (lastInst && instances.some((i) => i.name === lastInst)) playInstance.value = lastInst;
  const serverSelect = document.querySelector<HTMLSelectElement>("#play-server")!;
  serverSelect.innerHTML = `<option value="">None</option>` + servers.map((s) => `<option value="${s.address}">${s.name} — ${s.address}</option>`).join("");
}
invoke<Account[]>("list_accounts").then((a) => {
  renderAccounts(a);
  refreshPlaySelectors();
});
invoke<Instance[]>("list_instances").then((l) => {
  renderInstances(l);
  refreshPlaySelectors();
});
const playBtn = document.querySelector<HTMLButtonElement>("#play-btn")!;
const playStatus = document.querySelector<HTMLDivElement>("#play-status")!;
const launchBar = document.querySelector<HTMLProgressElement>("#launch-progress")!;

listen<{ phase: string; done: number }>("launch-progress", (e) => {
  launchBar.hidden = false;
  launchBar.value = e.payload.done;
  playStatus.textContent = `Launching… ${e.payload.phase} (${e.payload.done}%)`;
});

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
  if (inst.loader === "forge" || inst.loader === "neoforge") {
    playStatus.textContent = `${inst.loader === "forge" ? "Forge" : "NeoForge"} instances are not yet launchable: installer processors are still pending. Clone or recreate this instance with Fabric or Quilt.`;
    launchBar.hidden = true;
    return;
  }
  // Tokens live in the OS keychain, not in accounts.json.
  type StoredToken = { accessToken?: unknown };
  const readToken = (raw: string | null): string | undefined => {
    if (!raw) return undefined;
    let parsed: unknown;
    try {
      parsed = JSON.parse(raw);
    } catch {
      return undefined;
    }
    if (parsed && typeof parsed === "object" && "accessToken" in parsed) {
      // Named type (not inline shape): keychain blob written by addMicrosoftAccount.
      const t: StoredToken = parsed as StoredToken;
      return typeof t.accessToken === "string" ? t.accessToken : undefined;
    }
    return undefined;
  };
  let accessToken: string | undefined;
  if (acc.kind === "microsoft") {
    accessToken = readToken(await invoke<string | null>("get_account_token", { id: acc.id }));
    if (!accessToken) {
      playStatus.textContent = "Session expired — refreshing silently…";
      accessToken = await refreshMicrosoftAccount(acc.id) ?? undefined;
      if (!accessToken) {
        playStatus.textContent = "Microsoft token expired — remove and re-add the account.";
        return;
      }
    }
  }
  playStatus.textContent = `Launching ${inst.name} (${inst.game_version}) as ${acc.mc_name}…`;
  try {
    const serverAddr = (document.querySelector<HTMLSelectElement>("#play-server")!).value.trim();
    const r = await invoke<{ pid: number }>("launch_game", {
      version: inst.game_version,
      accountId: acc.id,
      accountName: acc.mc_name,
      accountUuid: acc.uuid,
      accessToken,
      instance: inst.name,
      loader: inst.loader,
      server: serverAddr || null,
    });
    launchBar.hidden = true;
    try {
      await invoke("touch_account", { id: acc.id });
      localStorage.setItem("rem-last-instance", inst.name);
      renderAccounts(await invoke<Account[]>("list_accounts"));
    } catch {
      // usage memory is best-effort
    }
  } catch (e) {
    playStatus.textContent = `Launch failed: ${e}`;
    launchBar.hidden = true;
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
function renderServers(servers: SavedServer[]) {
  serversEl.innerHTML = "";
  for (const s of servers) {
    const row = document.createElement("div");
    row.className = "card";
    const label = document.createElement("span");
    label.textContent = `${s.name} — ${s.address}${s.last_ping_ms == null ? "" : ` (${s.last_ping_ms} ms)`}`;
    const ping = document.createElement("button");
    ping.textContent = "Ping";
    ping.onclick = async () => {
      ping.disabled = true;
      try {
        const updated = await invoke<SavedServer>("ping_server", { id: s.id });
        label.textContent = `${updated.name} — ${updated.address} (${updated.last_ping_ms} ms)`;
      } catch (e) { label.textContent = `${s.name} — ping failed: ${e}`; }
      ping.disabled = false;
    };
    const remove = document.createElement("button");
    remove.textContent = "Remove";
    remove.className = "danger";
    remove.onclick = async () => {
      renderServers(await invoke<SavedServer[]>("remove_server", { id: s.id }));
      refreshPlaySelectors();
    };
    row.append(label, ping, remove);
    serversEl.appendChild(row);
  }
  if (!servers.length) serversEl.textContent = "No saved servers yet.";
}

addServerBtn.onclick = async () => {
  try {
    const list = await invoke<SavedServer[]>("add_server", { name: serverName.value.trim(), address: serverAddress.value.trim() });
    renderServers(list);
    refreshPlaySelectors();
    serverName.value = "";
    serverAddress.value = "";
  } catch (e) { serversEl.textContent = `Add failed: ${e}`; }
};

const pingAllBtn = document.querySelector<HTMLButtonElement>("#ping-all");
if (pingAllBtn) {
  pingAllBtn.onclick = async () => {
    const current = await invoke<SavedServer[]>("list_servers");
    if (!current.length) return;
    pingAllBtn.disabled = true;
    pingAllBtn.textContent = "Pinging…";
    for (const s of current) {
      try { await invoke("ping_server", { id: s.id }); } catch {}
    }
    renderServers(await invoke<SavedServer[]>("list_servers"));
    pingAllBtn.disabled = false;
    pingAllBtn.textContent = "Ping all";
  };
}

invoke<SavedServer[]>("list_servers").then(renderServers);
