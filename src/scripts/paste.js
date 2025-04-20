let textAreaEl = document.getElementById("text");
let statusEl = document.getElementById("status");

document.addEventListener("DOMContentLoaded", function () {
  textAreaEl = document.getElementById("text");
  statusEl = document.getElementById("status");
});

function scrollToTop() {
  window.scrollTo({ top: 0, behavior: "smooth" });
}

function scrollToBottom() {
  window.scrollTo({ top: document.body.scrollHeight, behavior: "smooth" });
}

function clearPaste() {
  const time = getTimeLabel();
  textAreaEl.value = "";
  statusEl.innerHTML = `[${time}] Data has been cleared (local only)`;
}

function copyToClipboard() {
  const time = getTimeLabel();
  textAreaEl.select();
  textAreaEl.setSelectionRange(0, 99999);

  navigator.clipboard.writeText(textAreaEl.value);
  statusEl.innerHTML = `[${time}] Data copied to clipboard`;
}

function savePaste() {
  fetch("/paste", {
    method: "POST",
    headers: {
      "Content-Type": "text/plain",
    },
    body: textAreaEl.value,
  })
    .then((response) => response.json())
    .then((data) => {
      const time = getTimeLabel();
      statusEl.innerHTML = `[${time}] ${data.message}`;
    })
    .catch((error) => {
      statusEl.innerHTML =
        `[${time}] ERROR: Failed to upload data to server: ` + error.message;
    });
}

function getTimeLabel() {
  const now = new Date();
  const h = String(now.getHours()).padStart(2, "0");
  const m = String(now.getMinutes()).padStart(2, "0");
  const s = String(now.getSeconds()).padStart(2, "0");
  return `${h}:${m}:${s}`;
}
