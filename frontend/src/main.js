const TOKEN_KEY = "token";
const USER_KEY = "coach_user";

const api = (path, opts = {}) => {
  const token = localStorage.getItem(TOKEN_KEY);
  const headers = new Headers(opts.headers || {});
  if (!headers.has("Content-Type") && opts.body) {
    headers.set("Content-Type", "application/json");
  }
  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }
  return fetch(path, { ...opts, headers });
};

const $ = (sel) => document.querySelector(sel);

let selectedTraineeId = null;

function setAuthMessage(text, ok = false) {
  const el = $("#auth-message");
  el.textContent = text || "";
  el.classList.toggle("ok", ok);
  if (!text) el.classList.remove("ok");
}

function showLogin() {
  $("#view-login").classList.remove("hidden");
  $("#view-app").classList.add("hidden");
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(USER_KEY);
  selectedTraineeId = null;
}

function showApp() {
  $("#view-login").classList.add("hidden");
  $("#view-app").classList.remove("hidden");
}

function loadUserFromStorage() {
  try {
    return JSON.parse(localStorage.getItem(USER_KEY) || "null");
  } catch {
    return null;
  }
}

function saveSession(token, user) {
  localStorage.setItem(TOKEN_KEY, token);
  localStorage.setItem(USER_KEY, JSON.stringify(user));
}

async function bootstrapSession() {
  const token = localStorage.getItem(TOKEN_KEY);
  if (!token) {
    showLogin();
    return;
  }
  const res = await api("/api/auth/me");
  if (!res.ok) {
    showLogin();
    return;
  }
  const user = await res.json();
  saveSession(token, user);
  $("#coach-email").textContent = user.email || "";
  showApp();
  await refreshTrainees();
}

function fmtDate(iso) {
  if (!iso) return "—";
  const d = new Date(iso);
  return d.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function renderTraineeList(trainees) {
  const ul = $("#trainee-list");
  ul.innerHTML = "";
  if (!trainees.length) {
    ul.innerHTML =
      '<li class="muted" style="padding:0.5rem 0">No trainees yet — add your first athlete.</li>';
    return;
  }
  for (const t of trainees) {
    const li = document.createElement("li");
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "trainee-item" + (selectedTraineeId === t.id ? " active" : "");
    const h = t.height_cm != null ? `${t.height_cm} cm` : "—";
    const w = t.weight_kg != null ? `${t.weight_kg} kg` : "—";
    btn.innerHTML = `<strong>${escapeHtml(t.display_name)}</strong><span class="sub">${h} · ${w}</span>`;
    btn.addEventListener("click", () => selectTrainee(t.id));
    li.appendChild(btn);
    ul.appendChild(li);
  }
}

function escapeHtml(s) {
  const d = document.createElement("div");
  d.textContent = s;
  return d.innerHTML;
}

async function refreshTrainees() {
  const res = await api("/api/trainees");
  if (!res.ok) return;
  const list = await res.json();
  renderTraineeList(list);
}

async function selectTrainee(id) {
  selectedTraineeId = id;
  await refreshTrainees();
  $("#detail-empty").classList.add("hidden");
  $("#detail-content").classList.remove("hidden");

  const res = await api(`/api/trainees/${id}`);
  if (!res.ok) {
    setAuthMessage("Could not load trainee.", false);
    return;
  }
  const data = await res.json();
  const { trainee, metrics, recent_sessions, volume_by_exercise, personal_records } = data;

  $("#dt-name").textContent = trainee.display_name;
  const parts = [];
  if (trainee.height_cm != null) parts.push(`${trainee.height_cm} cm`);
  if (trainee.weight_kg != null) parts.push(`${trainee.weight_kg} kg`);
  if (trainee.email) parts.push(trainee.email);
  $("#dt-meta").textContent = parts.join(" · ") || "No measurements yet";

  $("#et-name").value = trainee.display_name;
  $("#et-email").value = trainee.email || "";
  $("#et-height").value = trainee.height_cm ?? "";
  $("#et-weight").value = trainee.weight_kg ?? "";
  $("#et-notes").value = trainee.notes || "";

  if (metrics.length) {
    $("#metrics-table").innerHTML = `<table><thead><tr><th>Date</th><th>Weight</th><th>Height</th></tr></thead><tbody>${metrics
      .map(
        (m) =>
          `<tr><td>${fmtDate(m.recorded_at)}</td><td>${m.weight_kg} kg</td><td>${m.height_cm != null ? m.height_cm + " cm" : "—"}</td></tr>`,
      )
      .join("")}</tbody></table>`;
  } else {
    $("#metrics-table").innerHTML = '<p class="empty-table">No check-ins logged yet.</p>';
  }

  if (personal_records.length) {
    $("#prs-table").innerHTML = `<table><thead><tr><th>Exercise</th><th>Max (kg)</th></tr></thead><tbody>${personal_records
      .map((r) => `<tr><td>${escapeHtml(r.exercise_name)}</td><td>${r.max_weight_kg}</td></tr>`)
      .join("")}</tbody></table>`;
  } else {
    $("#prs-table").innerHTML = '<p class="empty-table">Log sessions with sets to see PRs.</p>';
  }

  if (volume_by_exercise.length) {
    $("#volume-table").innerHTML = `<table><thead><tr><th>Exercise</th><th>Total volume</th></tr></thead><tbody>${volume_by_exercise
      .map(
        (v) =>
          `<tr><td>${escapeHtml(v.exercise_name)}</td><td>${Math.round(v.total_volume)} kg·reps</td></tr>`,
      )
      .join("")}</tbody></table>`;
  } else {
    $("#volume-table").innerHTML = '<p class="empty-table">No volume data for this trainee yet.</p>';
  }

  const sess = $("#sessions-list");
  sess.innerHTML = "";
  if (recent_sessions.length) {
    for (const s of recent_sessions) {
      const li = document.createElement("li");
      li.innerHTML = `<span>${escapeHtml(s.title)}</span><span class="muted">${fmtDate(s.started_at)}</span>`;
      sess.appendChild(li);
    }
  } else {
    sess.innerHTML = '<li class="muted">No sessions yet.</li>';
  }
}

$("#register").addEventListener("click", async () => {
  setAuthMessage("");
  const res = await api("/api/auth/register", {
    method: "POST",
    body: JSON.stringify({
      email: $("#email").value.trim(),
      password: $("#password").value,
    }),
  });
  const body = await res.json().catch(() => ({}));
  if (res.ok && body.token) {
    saveSession(body.token, body.user);
    setAuthMessage("Account created — you’re in.", true);
    $("#coach-email").textContent = body.user?.email || "";
    showApp();
    await refreshTrainees();
  } else {
    setAuthMessage(body.error || `Could not register (${res.status})`);
  }
});

$("#login").addEventListener("click", async () => {
  setAuthMessage("");
  const res = await api("/api/auth/login", {
    method: "POST",
    body: JSON.stringify({
      email: $("#email").value.trim(),
      password: $("#password").value,
    }),
  });
  const body = await res.json().catch(() => ({}));
  if (res.ok && body.token) {
    saveSession(body.token, body.user);
    $("#coach-email").textContent = body.user?.email || "";
    showApp();
    await refreshTrainees();
  } else {
    setAuthMessage(body.error || `Login failed (${res.status})`);
  }
});

$("#logout").addEventListener("click", () => {
  showLogin();
  $("#detail-empty").classList.remove("hidden");
  $("#detail-content").classList.add("hidden");
  $("#trainee-list").innerHTML = "";
});

$("#toggle-add-trainee").addEventListener("click", () => {
  $("#form-add-trainee").classList.toggle("hidden");
});

$("#cancel-add-trainee").addEventListener("click", () => {
  $("#form-add-trainee").classList.add("hidden");
});

$("#form-add-trainee").addEventListener("submit", async (e) => {
  e.preventDefault();
  const name = $("#nt-name").value.trim();
  if (!name) return;
  const payload = {
    display_name: name,
    email: $("#nt-email").value.trim() || null,
    height_cm: numOrNull($("#nt-height").value),
    weight_kg: numOrNull($("#nt-weight").value),
    notes: $("#nt-notes").value.trim() || null,
  };
  const res = await api("/api/trainees", {
    method: "POST",
    body: JSON.stringify(payload),
  });
  if (res.ok) {
    $("#form-add-trainee").reset();
    $("#form-add-trainee").classList.add("hidden");
    await refreshTrainees();
  }
});

function numOrNull(v) {
  if (v === "" || v == null) return null;
  const n = Number(v);
  return Number.isFinite(n) ? n : null;
}

$("#toggle-edit-trainee").addEventListener("click", () => {
  $("#form-edit-trainee").classList.toggle("hidden");
});

$("#cancel-edit-trainee").addEventListener("click", () => {
  $("#form-edit-trainee").classList.add("hidden");
});

$("#form-edit-trainee").addEventListener("submit", async (e) => {
  e.preventDefault();
  if (!selectedTraineeId) return;
  const payload = {
    display_name: $("#et-name").value.trim() || null,
    email: $("#et-email").value.trim() || null,
    height_cm: numOrNull($("#et-height").value),
    weight_kg: numOrNull($("#et-weight").value),
    notes: $("#et-notes").value.trim() || null,
  };
  const res = await api(`/api/trainees/${selectedTraineeId}`, {
    method: "PATCH",
    body: JSON.stringify(payload),
  });
  if (res.ok) {
    $("#form-edit-trainee").classList.add("hidden");
    await selectTrainee(selectedTraineeId);
    await refreshTrainees();
  }
});

$("#btn-log-metric").addEventListener("click", async () => {
  if (!selectedTraineeId) return;
  const w = numOrNull($("#log-weight").value);
  if (w == null) return;
  const res = await api(`/api/trainees/${selectedTraineeId}/metrics`, {
    method: "POST",
    body: JSON.stringify({
      weight_kg: w,
      height_cm: numOrNull($("#log-height").value),
    }),
  });
  if (res.ok) {
    $("#log-weight").value = "";
    $("#log-height").value = "";
    await selectTrainee(selectedTraineeId);
    await refreshTrainees();
  }
});

$("#btn-create-workout").addEventListener("click", async () => {
  if (!selectedTraineeId) return;
  const title = $("#wo-title").value.trim() || "Session";
  const notes = $("#wo-notes").value.trim() || null;
  const res = await api("/api/workouts", {
    method: "POST",
    body: JSON.stringify({
      title,
      notes,
      trainee_id: selectedTraineeId,
    }),
  });
  if (res.ok) {
    $("#wo-title").value = "";
    $("#wo-notes").value = "";
    await selectTrainee(selectedTraineeId);
  }
});

bootstrapSession();
