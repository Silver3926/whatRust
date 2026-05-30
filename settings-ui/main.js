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

// --- Accounts ---

function renderAccounts(accounts) {
  const list = document.getElementById("accounts-list");
  list.textContent = "";
  const canRemove = accounts.length > 1;

  for (const a of accounts) {
    const row = document.createElement("div");
    row.className = "account-row";

    const name = document.createElement("span");
    name.className = "acct-name";
    name.textContent = a.name;
    row.appendChild(name);

    if (a.unread > 0) {
      const badge = document.createElement("span");
      badge.className = "acct-unread";
      badge.textContent = String(a.unread);
      row.appendChild(badge);
    }

    const openBtn = document.createElement("button");
    openBtn.className = "secondary";
    openBtn.textContent = "Open";
    openBtn.addEventListener("click", async () => {
      await invoke("open_account", { id: a.id });
    });
    row.appendChild(openBtn);

    const renameBtn = document.createElement("button");
    renameBtn.className = "secondary";
    renameBtn.textContent = "Rename";
    renameBtn.addEventListener("click", async () => {
      const next = prompt("Rename account", a.name);
      if (next && next.trim() && next.trim() !== a.name) {
        try {
          await invoke("rename_account", { id: a.id, name: next.trim() });
        } catch (e) {
          alert(String(e));
        }
        await loadAccounts();
      }
    });
    row.appendChild(renameBtn);

    const removeBtn = document.createElement("button");
    removeBtn.className = "secondary";
    removeBtn.textContent = "Remove";
    removeBtn.disabled = !canRemove;
    removeBtn.addEventListener("click", async () => {
      if (!confirm(`Remove account "${a.name}"? Its local session will be deleted.`)) return;
      try {
        await invoke("remove_account", { id: a.id });
      } catch (e) {
        alert(String(e));
      }
      await loadAccounts();
    });
    row.appendChild(removeBtn);

    list.appendChild(row);
  }
}

async function loadAccounts() {
  try {
    const accounts = await invoke("list_accounts");
    renderAccounts(accounts);
  } catch (e) {
    // ignore — listing failed
  }
}

async function addAccount() {
  const input = document.getElementById("new_account_name");
  const name = input.value.trim();
  if (!name) return;
  try {
    await invoke("add_account", { name });
    input.value = "";
    await loadAccounts();
  } catch (e) {
    // macOS < 14: disable add and surface the note.
    const note = document.getElementById("macos-note");
    note.textContent = String(e);
    note.hidden = false;
    document.getElementById("add_account").disabled = true;
    document.getElementById("new_account_name").disabled = true;
  }
}

window.addEventListener("DOMContentLoaded", () => {
  load();
  loadAccounts();
  document.getElementById("save").addEventListener("click", save);
  document.getElementById("add_account").addEventListener("click", addAccount);
  document.getElementById("new_account_name").addEventListener("keydown", (e) => {
    if (e.key === "Enter") addAccount();
  });
});
