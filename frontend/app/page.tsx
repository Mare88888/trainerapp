"use client";

import { useEffect, useState } from "react";

type HealthState = "loading" | "ok" | "error";

export default function Page() {
  const [health, setHealth] = useState<HealthState>("loading");

  useEffect(() => {
    let cancelled = false;
    fetch("/health")
      .then((r) => {
        if (cancelled) return;
        setHealth(r.ok ? "ok" : "error");
      })
      .catch(() => {
        if (cancelled) return;
        setHealth("error");
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <main style={{ maxWidth: 900, margin: "0 auto", padding: "2rem 1rem" }}>
      <section className="card">
        <h1 style={{ marginTop: 0 }}>CoachLift Frontend</h1>
        <p className="muted">
          Frontend is now Next.js + React (TypeScript). Rust backend remains the API source.
        </p>
        <p>
          Backend health:{" "}
          <strong>{health === "loading" ? "checking..." : health === "ok" ? "online" : "offline"}</strong>
        </p>
      </section>
    </main>
  );
}

