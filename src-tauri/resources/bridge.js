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
})();
