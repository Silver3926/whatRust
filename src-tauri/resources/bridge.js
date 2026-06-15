(function () {
  "use strict";
  if (window.location.origin !== "https://web.whatsapp.com") return;

  function invoke(cmd, args) {
    var t = window.__TAURI__;
    if (t && t.core && typeof t.core.invoke === "function") {
      return t.core.invoke(cmd, args).catch(function () {});
    }
    return Promise.resolve();
  }

  // 1) Client Hints shim — navigator.userAgentData is undefined in WebKitGTK,
  //    which WhatsApp's capability check can trip over.
  try {
    if (!navigator.userAgentData) {
      Object.defineProperty(navigator, "userAgentData", {
        configurable: true,
        value: {
          brands: [
            { brand: "Chromium", version: "131" },
            { brand: "Google Chrome", version: "131" },
            { brand: "Not_A Brand", version: "24" },
          ],
          mobile: false,
          platform: "Linux",
          getHighEntropyValues: function () {
            return Promise.resolve({
              platform: "Linux",
              platformVersion: "",
              architecture: "x86",
              model: "",
              uaFullVersion: "131.0.0.0",
            });
          },
        },
      });
    }
  } catch (e) {}

  // 2) Notification override — forward to a native OS notification via Rust.
  try {
    function ShimNotification(title, options) {
      options = options || {};
      this.title = title;
      this.body = options.body || "";
      this.onclick = null;
      this.onclose = null;
      this.onerror = null;
      this.onshow = null;
      invoke("notify", {
        title: String(title || "WhatsApp"),
        body: String(options.body || ""),
      });
    }
    ShimNotification.prototype.close = function () {
      if (typeof this.onclose === "function") this.onclose();
    };
    ShimNotification.prototype.addEventListener = function () {};
    ShimNotification.prototype.removeEventListener = function () {};
    ShimNotification.permission = "granted";
    ShimNotification.requestPermission = function (cb) {
      if (typeof cb === "function") cb("granted");
      return Promise.resolve("granted");
    };
    window.Notification = ShimNotification;
  } catch (e) {}

  // 3) Unread count — forward the raw <title> string on change; Rust parses it.
  var lastTitle = "";
  function report() {
    if (document.title === lastTitle) return;
    lastTitle = document.title;
    invoke("set_unread", { title: document.title });
  }
  function start() {
    try {
      var el = document.querySelector("title");
      if (el) {
        new MutationObserver(report).observe(el, {
          childList: true,
          characterData: true,
          subtree: true,
        });
      }
      setInterval(report, 2000); // fallback if <title> node is swapped
      report();
    } catch (e) {}
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }

  // 4) Drag-and-drop file injection.
  //    The native webview never delivers an OS file drop into this page on Linux
  //    (broken on Wayland; on X11 the GTK drop is accepted but never reaches the DOM),
  //    so Rust captures the drop (window.rs `register_drop_handler`) and calls this with
  //    the file bytes. We rebuild the File(s) and hand them to WhatsApp's own attach flow.
  //
  //    Primary: a hidden <input type=file> — `input.files` is settable in WebKit and
  //    React fires onChange for synthetic, bubbling events (it never checks isTrusted).
  //    Fallback: a synthetic drop. In WebKit `new DragEvent('drop',{dataTransfer})` drops
  //    the dataTransfer, so we dispatch a plain Event with `dataTransfer` defined via
  //    Object.defineProperty (the Cypress/Playwright WebKit workaround).
  //    WhatsApp opens a preview/Send composer (never auto-sends), so attempting both is safe.
  //    Every boundary is logged through the `dlog` command (see commands.rs).
  try {
    var drop = {};
    drop.log = function (m) {
      try {
        invoke("dlog", { msg: String(m).slice(0, 280) });
      } catch (e) {}
    };
    drop.b64ToFile = function (b64, name, type) {
      var bin = atob(b64),
        n = bin.length,
        u = new Uint8Array(n);
      for (var i = 0; i < n; i++) u[i] = bin.charCodeAt(i);
      return new File([u], name, { type: type || "application/octet-stream" });
    };
    drop.dataTransfer = function (files) {
      var dt = new DataTransfer();
      for (var i = 0; i < files.length; i++) dt.items.add(files[i]);
      return dt;
    };
    // WhatsApp opens a media preview / caption composer once a file is attached; use it
    // as the success signal. Selectors are best-effort across WhatsApp Web versions.
    drop.composerOpen = function () {
      return !!(
        document.querySelector('[data-testid="media-caption-input-container"]') ||
        document.querySelector('[data-testid="media-editor"]') ||
        document.querySelector('[data-animate-modal-body="true"]') ||
        document.querySelector('span[data-icon="send"]') ||
        document.querySelector('span[data-icon="media-cancel"]') ||
        document.querySelector('span[data-icon="x-viewer"]')
      );
    };
    drop.waitFor = function (pred, ms) {
      return new Promise(function (resolve) {
        var t0 = Date.now();
        (function poll() {
          if (pred()) return resolve(true);
          if (Date.now() - t0 >= ms) return resolve(false);
          setTimeout(poll, 80);
        })();
      });
    };
    drop.tryFileInput = function (dt) {
      var sels = [
        'input[type="file"][accept*="image"][accept*="video"]',
        'input[type="file"][accept*="image"]',
        'input[type="file"][accept]',
        'input[type="file"]',
      ];
      var input = null;
      for (var i = 0; i < sels.length && !input; i++) {
        input = document.querySelector(sels[i]);
      }
      if (!input) return false;
      try {
        input.files = dt.files; // settable in WebKit + Blink (WHATWG html#2861)
      } catch (e) {
        drop.log("input.files assign threw: " + e);
        return false;
      }
      input.dispatchEvent(new Event("change", { bubbles: true }));
      input.dispatchEvent(new Event("input", { bubbles: true }));
      drop.log("file-input path fired (" + dt.files.length + " file)");
      return true;
    };
    drop.tryDrop = function (dt, cssX, cssY) {
      var target = null;
      var el = document.elementFromPoint(cssX, cssY);
      if (el && el !== document.documentElement && el !== document.body) target = el;
      if (!target) {
        target =
          document.querySelector('[data-testid="conversation-panel-body"]') ||
          document.querySelector("div#main") ||
          document.querySelector("main") ||
          document.body;
      }
      var fire = function (type) {
        var ev = new Event(type, { bubbles: true, cancelable: true });
        try {
          Object.defineProperty(ev, "dataTransfer", { value: dt, configurable: true });
        } catch (e) {}
        target.dispatchEvent(ev);
      };
      fire("dragenter");
      fire("dragover");
      fire("drop");
      drop.log("synthetic-drop fired on " + (target.id || target.tagName));
    };
    // De-dupe: a given file can't be re-injected within 4s (guards eval retries).
    drop.seen = Object.create(null);
    drop.dedupe = function (list) {
      return list.filter(function (f) {
        var k = f.name + "|" + (f.b64 || "").slice(0, 24);
        if (drop.seen[k]) return false;
        drop.seen[k] = 1;
        setTimeout(function () {
          delete drop.seen[k];
        }, 4000);
        return true;
      });
    };

    window.__whatrustHandleDrop = function (fileObjs, physX, physY) {
      try {
        if (!Array.isArray(fileObjs) || fileObjs.length === 0) return;
        var fresh = drop.dedupe(fileObjs);
        if (fresh.length === 0) {
          drop.log("duplicate drop ignored");
          return;
        }
        var dpr = window.devicePixelRatio || 1;
        var cssX = physX / dpr,
          cssY = physY / dpr;
        var files = fresh.map(function (f) {
          return drop.b64ToFile(f.b64, f.name, f.type);
        });
        var dt = drop.dataTransfer(files);
        drop.log(
          "received " +
            files.length +
            " file(s) [" +
            files
              .map(function (f) {
                return f.type;
              })
              .join(",") +
            "]"
        );

        var dropFallback = function (reason) {
          drop.log("fallback to synthetic drop (" + reason + ")");
          drop.tryDrop(dt, cssX, cssY);
          drop.waitFor(drop.composerOpen, 1800).then(function (ok) {
            drop.log(ok ? "composer opened via synthetic drop" : "composer NOT detected after synthetic drop");
          });
        };

        if (drop.tryFileInput(dt)) {
          drop.waitFor(drop.composerOpen, 1500).then(function (ok) {
            if (ok) drop.log("composer opened via file-input");
            else dropFallback("file-input fired but composer not detected");
          });
        } else {
          dropFallback("no file input present");
        }
      } catch (e) {
        drop.log("handler error: " + e);
      }
    };
    drop.log("drop injector ready");
  } catch (e) {}
})();
