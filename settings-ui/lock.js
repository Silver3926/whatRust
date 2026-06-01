const invoke = window.__TAURI__.core.invoke;
const pwd = document.getElementById("password");
const errEl = document.getElementById("error");

async function tryUnlock() {
  const password = pwd.value;
  if (!password) return;
  errEl.textContent = "";
  try {
    const ok = await invoke("unlock_password", { password });
    if (!ok) {
      errEl.textContent = "Wrong password.";
      pwd.value = "";
      pwd.focus();
    }
  } catch (e) {
    errEl.textContent = String(e);
  }
}

async function tryBiometric() {
  errEl.textContent = "";
  try {
    const ok = await invoke("unlock_biometric");
    if (!ok) errEl.textContent = "Biometric authentication failed — enter your password.";
  } catch (e) {
    errEl.textContent = String(e);
  }
}

async function init() {
  document.getElementById("unlock").addEventListener("click", tryUnlock);
  pwd.addEventListener("keydown", (e) => { if (e.key === "Enter") tryUnlock(); });

  const bio = document.getElementById("biometric");
  bio.addEventListener("click", tryBiometric);

  document.getElementById("reset").addEventListener("click", async (e) => {
    e.preventDefault();
    const ok = confirm(
      "Reset whatRust?\n\nThis logs out ALL accounts and removes the app lock. " +
      "You will need to re-scan the WhatsApp QR code. Your chats stay on your phone."
    );
    if (!ok) return;
    try {
      await invoke("reset_app_lock");
    } catch (err) {
      errEl.textContent = String(err);
    }
  });

  try {
    const s = await invoke("get_lock_status");
    if (s.biometric_enabled) {
      bio.textContent = "Use " + s.biometric_label;
      bio.hidden = false;
      tryBiometric(); // auto-prompt on load
    }
  } catch (_) {
    // status unavailable — password still works
  }
  pwd.focus();
}

window.addEventListener("DOMContentLoaded", init);
