const invoke = window.__TAURI__?.core?.invoke;

const linksEl = document.querySelector("#links");
const destEl = document.querySelector("#destination");
const pickBtn = document.querySelector("#pick-dest");
const downloadBtn = document.querySelector("#download");
const statusEl = document.querySelector("#status");

function renderStatus(items) {
  statusEl.innerHTML = "";
  if (!items || items.length === 0) {
    const li = document.createElement("li");
    li.className = "empty";
    li.textContent = "还没有下载任务。";
    statusEl.appendChild(li);
    return;
  }
  for (const item of items) {
    const li = document.createElement("li");
    const head = document.createElement("div");
    const code = document.createElement("strong");
    code.className = "code";
    code.textContent = item.shortcode;
    const mark = document.createElement("span");
    mark.className = item.success ? "ok" : "bad";
    mark.textContent = item.success
      ? `  完成 · ${item.media_files.length} 个文件`
      : "  失败";
    head.append(code, mark);
    li.appendChild(head);
    const detail = document.createElement("p");
    detail.className = "files";
    if (item.success) {
      detail.textContent = item.media_files.join("\n");
    } else {
      detail.textContent = item.error || "instaloader 未能写入媒体文件";
    }
    li.appendChild(detail);
    statusEl.appendChild(li);
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  renderStatus([]);
  if (!invoke) {
    renderStatus([
      {
        shortcode: "应用",
        success: false,
        media_files: [],
        error: "Tauri API 不可用，请从桌面客户端启动。",
      },
    ]);
    downloadBtn.disabled = true;
    pickBtn.disabled = true;
    return;
  }
  try {
    destEl.value = await invoke("default_destination");
  } catch (err) {
    destEl.placeholder = String(err);
  }

  pickBtn.addEventListener("click", async () => {
    try {
      const folder = await invoke("pick_folder");
      if (folder) {
        destEl.value = folder;
      }
    } catch (err) {
      renderStatus([
        {
          shortcode: "目录",
          success: false,
          media_files: [],
          error: String(err),
        },
      ]);
    }
  });

  downloadBtn.addEventListener("click", async () => {
    downloadBtn.disabled = true;
    statusEl.innerHTML = "";
    const pending = document.createElement("li");
    pending.className = "empty";
    const lineCount = linksEl.value
      .split(/\r\n|\n|\r/)
      .map((line) => line.trim())
      .filter(Boolean).length;
    pending.textContent =
      lineCount > 1
        ? `正在按行下载 ${lineCount} 条链接…`
        : "正在通过 instaloader 下载…";
    statusEl.appendChild(pending);
    try {
      const results = await invoke("download_links", {
        links: linksEl.value,
        destination: destEl.value,
      });
      renderStatus(results);
    } catch (err) {
      renderStatus([
        {
          shortcode: "任务",
          success: false,
          media_files: [],
          error: String(err),
        },
      ]);
    } finally {
      downloadBtn.disabled = false;
    }
  });
});
