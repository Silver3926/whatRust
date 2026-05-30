const invoke = window.__TAURI__.core.invoke;
const BOOLS = ["close_to_tray", "start_minimized", "autostart", "notifications", "hotkey_enabled"];

async function load() {
  const s = await invoke("get_settings");
  for (const f of BOOLS) {
    const el = document.getElementById(f);
    if (el) el.checked = !!s[f];
  }
  document.getElementById("hotkey").value = s.hotkey || "CmdOrCtrl+Shift+W";
}

async function save() {
  const s = await invoke("get_settings");
  for (const f of BOOLS) {
    const el = document.getElementById(f);
    if (el) s[f] = el.checked;
  }
  const hk = document.getElementById("hotkey").value.trim();
  s.hotkey = hk || "CmdOrCtrl+Shift+W";
  await invoke("set_settings", { settings: s });
  const note = document.getElementById("note");
  note.textContent = "Saved ✓";
  setTimeout(() => (note.textContent = ""), 1500);
}

window.addEventListener("DOMContentLoaded", () => {
  load();
  document.getElementById("save").addEventListener("click", save);
});
