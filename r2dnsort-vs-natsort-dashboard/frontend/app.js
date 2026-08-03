const API = "";

const els = {
  statusPills: document.getElementById("statusPills"),
  sampleChips: document.getElementById("sampleChips"),
  itemsInput: document.getElementById("itemsInput"),
  itemCount: document.getElementById("itemCount"),
  algSelect: document.getElementById("algSelect"),
  reverseCheck: document.getElementById("reverseCheck"),
  runsInput: document.getElementById("runsInput"),
  flagChips: document.getElementById("flagChips"),
  runBtn: document.getElementById("runBtn"),
  matchBanner: document.getElementById("matchBanner"),
  speedupText: document.getElementById("speedupText"),
};

const activeFlags = new Set();
let samples = {};

function updateItemCount() {
  const n = els.itemsInput.value.split("\n").filter((l) => l.trim() !== "").length;
  els.itemCount.textContent = `${n} item${n === 1 ? "" : "s"}`;
}
els.itemsInput.addEventListener("input", updateItemCount);

async function loadStatus() {
  try {
    const res = await fetch(`${API}/api/status`);
    const data = await res.json();

    els.statusPills.innerHTML = "";
    for (const [name, info] of [["natsort", data.natsort], ["r2dnsort", data.r2dnsort]]) {
      const pill = document.createElement("span");
      pill.className = `pill ${info.available ? "ok" : "bad"}`;
      pill.textContent = info.available
        ? `${name} ${info.version || ""} ready`
        : `${name} not installed`;
      if (!info.available && info.error) pill.title = info.error;
      els.statusPills.appendChild(pill);
    }

    els.flagChips.innerHTML = "";
    data.flags.forEach((flag) => {
      const chip = document.createElement("span");
      chip.className = "chip";
      chip.textContent = flag;
      chip.addEventListener("click", () => {
        if (activeFlags.has(flag)) { activeFlags.delete(flag); chip.classList.remove("active"); }
        else { activeFlags.add(flag); chip.classList.add("active"); }
      });
      els.flagChips.appendChild(chip);
    });
  } catch (e) {
    els.statusPills.innerHTML = `<span class="pill bad">backend unreachable — is uvicorn running?</span>`;
  }
}

async function loadSamples() {
  try {
    const res = await fetch(`${API}/api/samples`);
    samples = await res.json();
    els.sampleChips.innerHTML = "";
    Object.keys(samples).forEach((key, i) => {
      const chip = document.createElement("span");
      chip.className = "chip";
      chip.textContent = key.replace(/_/g, " ");
      chip.addEventListener("click", () => {
        document.querySelectorAll("#sampleChips .chip").forEach((c) => c.classList.remove("active"));
        chip.classList.add("active");
        els.itemsInput.value = samples[key].join("\n");
        updateItemCount();
      });
      els.sampleChips.appendChild(chip);
      if (i === 0) chip.click();
    });
  } catch (e) { /* backend unreachable; handled by loadStatus */ }
}

function renderList(listEl, timeBadge, result, otherResultForDiff) {
  listEl.innerHTML = "";
  listEl.classList.remove("error");

  if (!result.available) {
    listEl.classList.add("error");
    listEl.innerHTML = `not installed on this machine — run:\n  pip install ${listEl.closest(".result-col").dataset.lib}`;
    timeBadge.textContent = "n/a";
    timeBadge.classList.add("err");
    return;
  }
  if (!result.ok) {
    listEl.classList.add("error");
    listEl.textContent = result.error || "error";
    timeBadge.textContent = "error";
    timeBadge.classList.add("err");
    return;
  }

  timeBadge.classList.remove("err");
  timeBadge.textContent = `${result.elapsed_ms_avg.toFixed(3)} ms avg`;

  result.result.forEach((val, i) => {
    const li = document.createElement("li");
    li.textContent = typeof val === "string" ? val : JSON.stringify(val);
    if (otherResultForDiff && otherResultForDiff[i] !== undefined) {
      const same = JSON.stringify(otherResultForDiff[i]) === JSON.stringify(val);
      if (!same) li.classList.add("diff");
    }
    listEl.appendChild(li);
  });
}

function renderBars(results) {
  const byLib = Object.fromEntries(results.map((r) => [r.library, r]));
  const times = results.filter((r) => r.ok).map((r) => r.elapsed_ms_avg);
  const max = Math.max(...times, 0.001);

  for (const lib of ["natsort", "r2dnsort"]) {
    const r = byLib[lib];
    const fill = document.getElementById(`bar-${lib}`);
    const val = document.getElementById(`barval-${lib}`);
    if (!r || !r.ok) {
      fill.style.width = "0%";
      val.textContent = "—";
      continue;
    }
    fill.style.width = `${(r.elapsed_ms_avg / max) * 100}%`;
    val.textContent = `${r.elapsed_ms_avg.toFixed(3)} ms`;
  }

  const py = byLib.natsort, rs = byLib.r2dnsort;
  if (py?.ok && rs?.ok) {
    const factor = py.elapsed_ms_avg / rs.elapsed_ms_avg;
    if (factor >= 1) {
      els.speedupText.innerHTML = `r2dnsort ran <span class="hl">${factor.toFixed(2)}×</span> faster than natsort on this input`;
    } else {
      els.speedupText.innerHTML = `natsort ran <span class="hl">${(1 / factor).toFixed(2)}×</span> faster than r2dnsort on this input`;
    }
  } else {
    els.speedupText.textContent = "";
  }
}

async function runSort() {
  const items = els.itemsInput.value.split("\n").filter((l) => l.trim() !== "");
  if (items.length === 0) return;

  els.runBtn.disabled = true;
  els.runBtn.textContent = "⏳ running…";

  const payload = {
    items,
    algorithm: els.algSelect.value,
    reverse: els.reverseCheck.checked,
    flags: Array.from(activeFlags),
    libraries: ["natsort", "r2dnsort"],
    runs: Math.min(200, Math.max(1, parseInt(els.runsInput.value || "1", 10))),
  };

  try {
    const res = await fetch(`${API}/api/sort`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    const data = await res.json();
    const byLib = Object.fromEntries(data.results.map((r) => [r.library, r]));

    renderList(
      document.getElementById("list-natsort"),
      document.getElementById("time-natsort"),
      byLib.natsort,
      byLib.r2dnsort?.result
    );
    renderList(
      document.getElementById("list-r2dnsort"),
      document.getElementById("time-r2dnsort"),
      byLib.r2dnsort,
      byLib.natsort?.result
    );

    const banner = els.matchBanner;
    banner.classList.remove("match", "mismatch");
    if (data.match === true) {
      banner.classList.add("show", "match");
      banner.textContent = `✓ Identical output — both libraries agree on all ${data.input_count} items`;
    } else if (data.match === false) {
      banner.classList.add("show", "mismatch");
      banner.textContent = `⚠ Outputs differ — mismatched lines highlighted below`;
    } else {
      banner.classList.remove("show");
    }

    renderBars(data.results);
  } catch (e) {
    els.matchBanner.classList.add("show", "mismatch");
    els.matchBanner.textContent = "Could not reach backend — make sure uvicorn is running.";
  } finally {
    els.runBtn.disabled = false;
    els.runBtn.textContent = "▶ Run both libraries";
  }
}

els.runBtn.addEventListener("click", runSort);

(async function init() {
  await Promise.all([loadStatus(), loadSamples()]);
  updateItemCount();
  runSort();
})();
