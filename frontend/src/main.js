const TOKEN_KEY = "token";
const USER_KEY = "coach_user";

const authState = {
  token: localStorage.getItem(TOKEN_KEY),
  user: (() => {
    try {
      return JSON.parse(localStorage.getItem(USER_KEY) || "null");
    } catch {
      return null;
    }
  })(),
};

const api = async (path, opts = {}) => {
  const token = authState.token;
  const headers = new Headers(opts.headers || {});
  if (!headers.has("Content-Type") && opts.body) {
    headers.set("Content-Type", "application/json");
  }
  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }
  const res = await fetch(path, { ...opts, headers });

  // Global auth guard for protected endpoints.
  if (opts.authRequired && res.status === 401) {
    clearSession();
    showLogin();
    setAuthMessage("Session expired. Please log in again.");
  }
  return res;
};

const $ = (sel) => document.querySelector(sel);

let selectedTraineeId = null;
let activeWorkoutId = null;
/** Workout opened from the sessions list (detail panel). */
let viewingWorkoutId = null;
const selectedWorkoutExercises = [];
let exerciseLibraryRows = [];

function setAuthMessage(text, ok = false) {
  const el = $("#auth-message");
  el.textContent = text || "";
  el.classList.toggle("ok", ok);
  if (!text) el.classList.remove("ok");
}

function showSignupPanel() {
  $("#panel-signup").classList.remove("hidden");
  $("#panel-login").classList.add("hidden");
  $("#form-login")?.reset();
  setAuthMessage("");
}

function showLoginPanel() {
  $("#panel-signup").classList.add("hidden");
  $("#panel-login").classList.remove("hidden");
  $("#form-signup")?.reset();
  setAuthMessage("");
}

function formatAuthError(res, body) {
  if (body?.error === "validation failed" && body.details && typeof body.details === "object") {
    const msgs = [];
    for (const key of Object.keys(body.details)) {
      const arr = body.details[key];
      if (Array.isArray(arr)) {
        for (const item of arr) {
          if (typeof item === "string") msgs.push(item);
          else if (item && typeof item.message === "string") msgs.push(item.message);
        }
      }
    }
    if (msgs.length) return msgs.join(" ");
  }
  if (typeof body?.error === "string" && body.error) return body.error;
  return `Something went wrong (${res.status}).`;
}

function showLogin() {
  $("#view-login").classList.remove("hidden");
  $("#view-app").classList.add("hidden");
  selectedTraineeId = null;
  showSignupPanel();
}

function showApp() {
  $("#view-login").classList.add("hidden");
  $("#view-app").classList.remove("hidden");
}

function saveSession(token, user) {
  authState.token = token;
  authState.user = user;
  localStorage.setItem(TOKEN_KEY, token);
  localStorage.setItem(USER_KEY, JSON.stringify(user));
}

function clearSession() {
  authState.token = null;
  authState.user = null;
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(USER_KEY);
}

function onAuthSuccess(token, user) {
  saveSession(token, user);
  setCoachHeader(user);
  showApp();
}

function setCoachHeader(user) {
  const first = (user?.first_name || "").trim();
  const last = (user?.last_name || "").trim();
  const full = [first, last].filter(Boolean).join(" ").trim();
  const nameEl = $("#coach-display-name");
  if (!nameEl) return;
  if (full) nameEl.textContent = full;
  else if (user?.id) nameEl.textContent = "Coach";
  else nameEl.textContent = "";
}

async function bootstrapSession() {
  const token = authState.token;
  if (!token) {
    showLogin();
    return;
  }
  const res = await api("/api/auth/me", { authRequired: true });
  if (!res.ok) {
    clearSession();
    showLogin();
    return;
  }
  const user = await res.json();
  saveSession(token, user);
  setCoachHeader(user);
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
    const age = t.age != null ? `${t.age}y` : "—";
    const h = t.height_cm != null ? `${t.height_cm} cm` : "—";
    const w = t.weight_kg != null ? `${t.weight_kg} kg` : "—";
    btn.innerHTML = `<strong>${escapeHtml(t.display_name)}</strong><span class="sub">${age} · ${h} · ${w}</span>`;
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
  const res = await api("/api/trainees", { authRequired: true });
  if (!res.ok) return;
  const list = await res.json();
  renderTraineeList(list);
}

async function loadTraineeWorkouts(traineeId, fallbackSessions = []) {
  const sess = $("#sessions-list");
  if (!sess) return;
  const res = await api(`/api/trainees/${traineeId}/workouts`, { authRequired: true });
  if (!res.ok) {
    renderWorkoutList(
      fallbackSessions.map((s) => ({
        id: s.id,
        title: s.title,
        started_at: s.started_at,
      })),
    );
    return;
  }
  const workouts = await res.json();
  const normalized = workouts.map((w) => ({
    id: w.id,
    title: w.title,
    started_at: w.started_at,
  }));
  renderWorkoutList(normalized);
}

function renderWorkoutList(items) {
  const sess = $("#sessions-list");
  sess.innerHTML = "";
  if (items.length) {
    for (const s of items) {
      const li = document.createElement("li");
      if (s.id) {
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = "session-open-btn";
        btn.innerHTML = `<span>${escapeHtml(s.title)}</span><span class="muted">${fmtDate(s.started_at)}</span>`;
        btn.addEventListener("click", () => openWorkoutDetail(s.id, s.title, s.started_at));
        li.appendChild(btn);
      } else {
        li.innerHTML = `<span>${escapeHtml(s.title)}</span><span class="muted">${fmtDate(s.started_at)}</span>`;
      }
      sess.appendChild(li);
    }
  } else {
    sess.innerHTML = '<li class="muted">No workouts yet.</li>';
  }
}

async function refreshWorkoutExercisesList() {
  const ul = $("#wd-exercises");
  if (!ul || !viewingWorkoutId) return;
  const res = await api(`/api/workouts/${viewingWorkoutId}/exercises`, { authRequired: true });
  if (!res.ok) {
    ul.innerHTML = '<li class="muted">Could not load exercises.</li>';
    return;
  }
  const list = await res.json();
  if (!list.length) {
    ul.innerHTML = '<li class="muted">No exercises in this workout yet.</li>';
    return;
  }
  ul.innerHTML = list
    .map(
      (ex) =>
        `<li><span class="wd-ex-name">${escapeHtml(ex.exercise?.name || "Exercise")}</span><span class="muted">${(ex.sets || []).length} sets</span></li>`,
    )
    .join("");
}

async function openWorkoutDetail(workoutId, title, startedAt) {
  viewingWorkoutId = workoutId;
  activeWorkoutId = workoutId;
  const titleInput = $("#wo-title");
  if (titleInput) titleInput.value = title || "";
  $("#wd-title").textContent = title || "Workout";
  $("#wd-meta").textContent = startedAt ? fmtDate(startedAt) : "";
  $("#workout-detail-panel")?.classList.remove("hidden");
  await refreshWorkoutExercisesList();
  await syncSelectedExercisesFromWorkout();
}

function closeWorkoutDetail() {
  viewingWorkoutId = null;
  $("#workout-detail-panel")?.classList.add("hidden");
}

async function selectTrainee(id) {
  const changedTrainee = selectedTraineeId !== id;
  selectedTraineeId = id;
  if (changedTrainee) {
    activeWorkoutId = null;
    viewingWorkoutId = null;
    selectedWorkoutExercises.length = 0;
    renderSelectedExercises();
    $("#exercise-picker")?.classList.add("hidden");
    closeWorkoutDetail();
  }
  await refreshTrainees();
  $("#detail-empty").classList.add("hidden");
  $("#detail-content").classList.remove("hidden");

  const res = await api(`/api/trainees/${id}`, { authRequired: true });
  if (!res.ok) {
    setAuthMessage("Could not load trainee.", false);
    return;
  }
  const data = await res.json();
  const { trainee, metrics, recent_sessions, volume_by_exercise, personal_records } = data;

  $("#dt-name").textContent = trainee.display_name;
  const parts = [];
  if (trainee.age != null) parts.push(`${trainee.age} years`);
  if (trainee.height_cm != null) parts.push(`${trainee.height_cm} cm`);
  if (trainee.weight_kg != null) parts.push(`${trainee.weight_kg} kg`);
  if (trainee.email) parts.push(trainee.email);
  $("#dt-meta").textContent = parts.join(" · ") || "No measurements yet";
  $("#basic-stats").innerHTML = `
    <div class="stat-card"><span class="label">Age</span><span class="value">${
      trainee.age != null ? `${trainee.age} years` : "—"
    }</span></div>
    <div class="stat-card"><span class="label">Height</span><span class="value">${
      trainee.height_cm != null ? `${trainee.height_cm} cm` : "—"
    }</span></div>
    <div class="stat-card"><span class="label">Weight</span><span class="value">${
      trainee.weight_kg != null ? `${trainee.weight_kg} kg` : "—"
    }</span></div>
  `;

  $("#et-name").value = trainee.display_name;
  $("#et-email").value = trainee.email || "";
  $("#et-age").value = trainee.age ?? "";
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

  await loadTraineeWorkouts(id, recent_sessions);
}

function renderSelectedExercises() {
  const host = $("#selected-exercises");
  if (!host) return;
  if (!selectedWorkoutExercises.length) {
    host.innerHTML =
      '<p class="empty-table" style="padding:1rem 1.1rem;">No exercises yet. Tap <strong>+ Exercise</strong>.</p>';
    return;
  }
  host.innerHTML = selectedWorkoutExercises
    .map((e) => {
      const sets = Array.isArray(e.sets) ? e.sets : [];
      const doneRows = sets
        .map((s, idx) => {
          const prev =
            idx === 0
              ? "—"
              : `${sets[idx - 1].weight_kg} × ${sets[idx - 1].reps}`;
          return `<tr class="done-row">
            <td>${s.set_number}</td>
            <td>${escapeHtml(String(prev))}</td>
            <td>${s.weight_kg}</td>
            <td>${s.reps}</td>
            <td class="hevy-set-check"><span class="hevy-done-mark" title="Logged">✓</span></td>
          </tr>`;
        })
        .join("");
      const last = sets[sets.length - 1];
      const nextNum = last ? last.set_number + 1 : 1;
      const prevForNew = last ? `${last.weight_kg} × ${last.reps}` : "—";
      const wid = e.workoutExerciseId;
      return `<article class="hevy-exercise-card" data-we-id="${wid}">
        <div class="hevy-exercise-head">
          <h4 class="hevy-exercise-name">${escapeHtml(e.name)}</h4>
        </div>
        <div class="hevy-set-table-wrap">
          <table class="hevy-set-table">
            <thead>
              <tr>
                <th>Set</th>
                <th>Previous</th>
                <th>kg</th>
                <th>Reps</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              ${doneRows}
              <tr class="hevy-add-set-row">
                <td>${nextNum}</td>
                <td>${escapeHtml(prevForNew)}</td>
                <td><input type="number" min="0" step="0.5" class="hevy-cell-input" placeholder="0" data-weight="${wid}" /></td>
                <td><input type="number" min="0" step="1" class="hevy-cell-input" placeholder="0" data-reps="${wid}" /></td>
                <td class="hevy-set-check hevy-set-check--empty" aria-hidden="true"></td>
              </tr>
            </tbody>
          </table>
        </div>
        <button type="button" class="hevy-add-set-btn" data-add-set="${wid}">
          <span class="hevy-add-set-icon" aria-hidden="true">+</span>
          Add Set
        </button>
      </article>`;
    })
    .join("");

  host.querySelectorAll("[data-add-set]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const id = btn.getAttribute("data-add-set");
      const card = host.querySelector(`.hevy-exercise-card[data-we-id="${id}"]`);
      const repsRaw = card?.querySelector(`[data-reps="${id}"]`)?.value?.trim() ?? "";
      const weightRaw = card?.querySelector(`[data-weight="${id}"]`)?.value?.trim() ?? "";
      let reps = Number(repsRaw);
      let weight = Number(weightRaw);
      // API requires reps >= 0 and weight >= 0; empty fields default to 0.
      if (!Number.isFinite(reps) || reps < 0) reps = 0;
      if (!Number.isFinite(weight) || weight < 0) weight = 0;
      const res = await api(`/api/workout-exercises/${id}/sets`, {
        method: "POST",
        authRequired: true,
        body: JSON.stringify({ reps, weight_kg: weight, is_warmup: false }),
      });
      if (res.ok) {
        await syncSelectedExercisesFromWorkout();
      } else {
        setAuthMessage("Could not add set.", false);
      }
    });
  });
}

async function syncSelectedExercisesFromWorkout() {
  if (!activeWorkoutId) return;
  const res = await api(`/api/workouts/${activeWorkoutId}`, { authRequired: true });
  if (!res.ok) return;
  const detail = await res.json();
  selectedWorkoutExercises.length = 0;
  for (const ex of detail.exercises || []) {
    selectedWorkoutExercises.push({
      workoutExerciseId: ex.workout_exercise_id,
      name: ex.exercise?.name || "Exercise",
      sets: Array.isArray(ex.sets) ? ex.sets : [],
    });
  }
  renderSelectedExercises();
  if (viewingWorkoutId && activeWorkoutId === viewingWorkoutId) {
    await refreshWorkoutExercisesList();
  }
}

async function ensureWorkoutSession() {
  if (!selectedTraineeId) return null;
  if (activeWorkoutId) return activeWorkoutId;
  const title = $("#wo-title").value.trim() || "Session";
  const res = await api(`/api/trainees/${selectedTraineeId}/workouts`, {
    method: "POST",
    authRequired: true,
    body: JSON.stringify({ title, notes: null }),
  });
  if (!res.ok) return null;
  const workout = await res.json();
  activeWorkoutId = workout.id;
  await loadTraineeWorkouts(selectedTraineeId);
  return activeWorkoutId;
}

async function loadExerciseLibrary() {
  const muscle = $("#exercise-muscle")?.value || "";
  const q = $("#exercise-search")?.value.trim() || "";
  const qs = new URLSearchParams();
  if (muscle) qs.set("muscle", muscle);
  if (q) qs.set("q", q);
  const path = qs.toString() ? `/api/exercises?${qs.toString()}` : "/api/exercises";
  const res = await api(path, { authRequired: true });
  const host = $("#exercise-results");
  if (!res.ok) {
    if (host) {
      host.innerHTML =
        '<p class="empty-table" style="padding:0.6rem;">Could not load exercises. Restart backend and try again.</p>';
    }
    return;
  }
  const rows = await res.json();
  exerciseLibraryRows = rows;
  populateExerciseSelect(rows);
  if (!host) return;
  host.innerHTML = rows.length
    ? rows
        .map(
          (r) => `
        <div class="exercise-result-row">
          <div>
            <strong>${escapeHtml(r.name)}</strong><br />
            <span class="muted">${escapeHtml(r.muscle || "general")}</span>
          </div>
          <button type="button" class="btn-hevy-secondary sm" data-add-ex="${encodeURIComponent(r.name)}">Add</button>
        </div>`,
        )
        .join("")
    : '<p class="empty-table" style="padding:0.6rem;">No exercises found.</p>';

  host.querySelectorAll("[data-add-ex]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const name = decodeURIComponent(btn.getAttribute("data-add-ex"));
      await addExerciseByName(name);
    });
  });
}

function populateExerciseSelect(rows) {
  const select = $("#exercise-select");
  if (!select) return;
  select.innerHTML = '<option value="">Choose exercise...</option>';
  for (const row of rows) {
    const opt = document.createElement("option");
    opt.value = row.name;
    opt.textContent = `${row.name} (${row.muscle || "general"})`;
    select.appendChild(opt);
  }
  if (rows.length > 0) {
    select.value = rows[0].name;
  }
}

async function addExerciseByName(name) {
  if (!name) return;
  const workoutId = await ensureWorkoutSession();
  if (!workoutId) return;
  const add = await api(`/api/workouts/${workoutId}/exercises`, {
    method: "POST",
    authRequired: true,
    body: JSON.stringify({ name, notes: null }),
  });
  if (!add.ok) return;
  await syncSelectedExercisesFromWorkout();
  await loadTraineeWorkouts(selectedTraineeId);
  if (viewingWorkoutId === workoutId) await refreshWorkoutExercisesList();
}

$("#goto-login").addEventListener("click", () => {
  showLoginPanel();
});

$("#goto-signup").addEventListener("click", () => {
  showSignupPanel();
});

$("#form-signup").addEventListener("submit", async (e) => {
  e.preventDefault();
  setAuthMessage("");
  const firstName = $("#signup-first-name").value.trim();
  const lastName = $("#signup-last-name").value.trim();
  const email = $("#signup-email").value.trim();
  const password = $("#signup-password").value;

  if (!firstName || !lastName) {
    setAuthMessage("Please enter your first and last name.");
    return;
  }
  if (!email) {
    setAuthMessage("Please enter your email.");
    return;
  }
  if (password.length < 8) {
    setAuthMessage("Password must be at least 8 characters.");
    return;
  }

  const res = await api("/api/auth/register", {
    method: "POST",
    body: JSON.stringify({
      first_name: firstName,
      last_name: lastName,
      email,
      password,
    }),
  });
  const body = await res.json().catch(() => ({}));
  if (res.ok && body.token) {
    setAuthMessage("Account created — you’re in.", true);
    onAuthSuccess(body.token, body.user);
    await refreshTrainees();
  } else {
    setAuthMessage(formatAuthError(res, body));
  }
});

$("#form-login").addEventListener("submit", async (e) => {
  e.preventDefault();
  setAuthMessage("");
  const email = $("#login-email").value.trim();
  const password = $("#login-password").value;

  if (!email) {
    setAuthMessage("Please enter your email.");
    return;
  }
  if (!password) {
    setAuthMessage("Please enter your password.");
    return;
  }

  const res = await api("/api/auth/login", {
    method: "POST",
    body: JSON.stringify({ email, password }),
  });
  const body = await res.json().catch(() => ({}));
  if (res.ok && body.token) {
    onAuthSuccess(body.token, body.user);
    await refreshTrainees();
  } else {
    setAuthMessage(formatAuthError(res, body));
  }
});

$("#logout").addEventListener("click", () => {
  clearSession();
  setCoachHeader({});
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
    age: intOrNull($("#nt-age").value),
    height_cm: numOrNull($("#nt-height").value),
    weight_kg: numOrNull($("#nt-weight").value),
    notes: $("#nt-notes").value.trim() || null,
  };
  const res = await api("/api/trainees", {
    method: "POST",
    body: JSON.stringify(payload),
    authRequired: true,
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

function intOrNull(v) {
  if (v === "" || v == null) return null;
  const n = Number(v);
  if (!Number.isFinite(n)) return null;
  const i = Math.trunc(n);
  return i > 0 ? i : null;
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
    age: intOrNull($("#et-age").value),
    height_cm: numOrNull($("#et-height").value),
    weight_kg: numOrNull($("#et-weight").value),
    notes: $("#et-notes").value.trim() || null,
  };
  const res = await api(`/api/trainees/${selectedTraineeId}`, {
    method: "PUT",
    body: JSON.stringify(payload),
    authRequired: true,
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
    authRequired: true,
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
  await ensureWorkoutSession();
});

$("#btn-open-exercise-picker").addEventListener("click", async () => {
  const picker = $("#exercise-picker");
  picker.classList.toggle("hidden");
  if (!picker.classList.contains("hidden")) {
    await loadExerciseLibrary();
  }
});

$("#btn-load-exercises").addEventListener("click", async () => {
  await loadExerciseLibrary();
});

$("#exercise-muscle").addEventListener("change", async () => {
  // Choosing a muscle group immediately narrows to that group.
  const search = $("#exercise-search");
  if (search) search.value = "";
  await loadExerciseLibrary();
});

$("#btn-add-selected-exercise").addEventListener("click", async () => {
  const selectedName = $("#exercise-select")?.value || "";
  await addExerciseByName(selectedName);
});

$("#wd-close").addEventListener("click", () => closeWorkoutDetail());

$("#form-wd-add-exercise").addEventListener("submit", async (e) => {
  e.preventDefault();
  const name = $("#wd-ex-name")?.value?.trim() || "";
  if (!name || !viewingWorkoutId) return;
  const res = await api(`/api/workouts/${viewingWorkoutId}/exercises`, {
    method: "POST",
    authRequired: true,
    body: JSON.stringify({ name, notes: null }),
  });
  if (!res.ok) return;
  $("#wd-ex-name").value = "";
  activeWorkoutId = viewingWorkoutId;
  await refreshWorkoutExercisesList();
  await syncSelectedExercisesFromWorkout();
  await loadTraineeWorkouts(selectedTraineeId);
});

bootstrapSession();
