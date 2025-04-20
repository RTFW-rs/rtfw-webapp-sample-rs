document.addEventListener("DOMContentLoaded", function () {
  const statusElem = document.getElementById("status");
  const formElem = document.querySelector("form");
  const downloadButtons = document.querySelectorAll("button.view-file");
  const deleteButtons = document.querySelectorAll("button.delete-file");

  downloadButtons.forEach((button) => {
    button.addEventListener("click", function (_event) {
      const fileUrl = button.dataset.url;
      globalThis.location.href = fileUrl;
    });
  });

  deleteButtons.forEach((button) => {
    button.addEventListener("click", function (_event) {
      const filename = button.dataset.filename;
      if (confirm(`Delete ${filename} forever?`)) {
        fetch("/files/", {
          method: "DELETE",
          body: filename,
        })
          .then((response) => response.json())
          .then((data) => {
            const time = getTimeLabel();
            statusElem.innerHTML = `[${time}] ${data.message}`;
            if (data.status.startsWith("2") === false) {
              // nothing here
            } else {
              globalThis.location.reload();
            }
          })
          .catch((error) => {
            statusElem.innerHTML =
              `[${time}] ERROR: Failed to delete file: ` + error.message;
          });
      }
    });
  });

  formElem.addEventListener("submit", function (event) {
    event.preventDefault();
    const formData = new FormData(formElem);
    fetch("/send", {
      method: "POST",
      body: formData,
    })
      .then((response) => response.json())
      .then((data) => {
        const time = getTimeLabel();
        statusElem.innerHTML = `[${time}] ${data.message}`;
        if (data.status.startsWith("2") === false) {
          // nothing here
        } else {
          globalThis.location.reload();
        }
      })
      .catch((error) => {
        statusElem.innerHTML =
          `[${time}] ERROR: Failed to upload data to server: ` + error.message;
      });
  });
});

function getTimeLabel() {
  const now = new Date();
  const h = String(now.getHours()).padStart(2, "0");
  const m = String(now.getMinutes()).padStart(2, "0");
  const s = String(now.getSeconds()).padStart(2, "0");
  return `${h}:${m}:${s}`;
}
