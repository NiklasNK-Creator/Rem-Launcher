import { invoke } from "@tauri-apps/api/core";

type Project = { id: string; title: string; description: string; downloads: number };

const q = document.querySelector<HTMLInputElement>("#q")!;
const go = document.querySelector<HTMLButtonElement>("#go")!;
const results = document.querySelector<HTMLDivElement>("#results")!;

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
