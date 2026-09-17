const statusEl = document.getElementById("status") as HTMLParagraphElement;

window.__TAURI__.core
  .invoke<string>("ping")
  .then((reply) => {
    statusEl.textContent = `ping -> ${reply}`;
  })
  .catch((err: unknown) => {
    statusEl.textContent = `ping failed: ${String(err)}`;
  });
