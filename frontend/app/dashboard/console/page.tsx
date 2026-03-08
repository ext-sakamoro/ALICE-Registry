"use client";
import { useState } from "react";

type Tab = "push" | "pull" | "diff" | "search" | "stats";

const DEFAULTS: Record<Tab, string> = {
  push: JSON.stringify(
    {
      repository: "alice/runtime",
      tag: "v1.2.0",
      manifest: {
        schema_version: 2,
        media_type: "application/vnd.oci.image.manifest.v1+json",
        config: {
          digest: "sha256:abc123",
          size: 1024,
        },
        layers: [
          { digest: "sha256:layer001", size: 52428800, media_type: "application/vnd.oci.image.layer.v1.tar+gzip" },
          { digest: "sha256:layer002", size: 10485760, media_type: "application/vnd.oci.image.layer.v1.tar+gzip" },
        ],
      },
    },
    null,
    2
  ),
  pull: JSON.stringify(
    {
      repository: "alice/runtime",
      reference: "v1.2.0",
      resolve_layers: true,
    },
    null,
    2
  ),
  diff: JSON.stringify(
    {
      repository: "alice/runtime",
      tag_a: "v1.1.0",
      tag_b: "v1.2.0",
      include_config_diff: true,
    },
    null,
    2
  ),
  search: JSON.stringify(
    {
      query: "alice runtime",
      filters: {
        namespace: "alice",
        label: "env=production",
      },
      limit: 20,
      sort_by: "push_time",
    },
    null,
    2
  ),
  stats: JSON.stringify({}, null, 2),
};

const ENDPOINTS: Record<Tab, { method: string; path: string }> = {
  push: { method: "POST", path: "/api/v1/registry/push" },
  pull: { method: "POST", path: "/api/v1/registry/pull" },
  diff: { method: "POST", path: "/api/v1/registry/diff" },
  search: { method: "POST", path: "/api/v1/registry/search" },
  stats: { method: "GET", path: "/api/v1/stats" },
};

const TAB_LABELS: Record<Tab, string> = {
  push: "Push",
  pull: "Pull",
  diff: "Diff",
  search: "Search",
  stats: "Stats",
};

export default function ConsolePage() {
  const [activeTab, setActiveTab] = useState<Tab>("push");
  const [inputs, setInputs] = useState<Record<Tab, string>>(DEFAULTS);
  const [response, setResponse] = useState("");
  const [loading, setLoading] = useState(false);

  const API = "http://localhost:8081";

  const send = async () => {
    setLoading(true);
    setResponse("");
    const { method, path } = ENDPOINTS[activeTab];
    try {
      const opts: RequestInit = { method, headers: { "Content-Type": "application/json" } };
      if (method === "POST") opts.body = inputs[activeTab];
      const res = await fetch(`${API}${path}`, opts);
      setResponse(JSON.stringify(await res.json(), null, 2));
    } catch (e: unknown) {
      setResponse(`Error: ${e instanceof Error ? e.message : String(e)}`);
    }
    setLoading(false);
  };

  const tabStyle = (tab: Tab): React.CSSProperties => ({
    padding: "8px 16px",
    border: "none",
    borderRadius: 6,
    cursor: "pointer",
    fontFamily: "monospace",
    fontWeight: activeTab === tab ? 700 : 400,
    background: activeTab === tab ? "#a78bfa" : "#1a1a2e",
    color: activeTab === tab ? "#000" : "#aaa",
  });

  return (
    <div style={{ padding: 24, fontFamily: "monospace", background: "#0a0a0a", minHeight: "100vh", color: "#fff" }}>
      <h1 style={{ color: "#a78bfa", marginBottom: 8 }}>ALICE Registry — Console</h1>
      <p style={{ color: "#666", marginBottom: 24, fontSize: 14 }}>
        Container and artifact registry SaaS · API: {API}
      </p>

      <div style={{ display: "flex", gap: 8, marginBottom: 16, flexWrap: "wrap" }}>
        {(Object.keys(TAB_LABELS) as Tab[]).map((tab) => (
          <button key={tab} style={tabStyle(tab)} onClick={() => setActiveTab(tab)}>
            {TAB_LABELS[tab]}
          </button>
        ))}
      </div>

      <div style={{ marginBottom: 8, fontSize: 12, color: "#666" }}>
        {ENDPOINTS[activeTab].method} {ENDPOINTS[activeTab].path}
      </div>

      <textarea
        value={inputs[activeTab]}
        onChange={(e) => setInputs((prev) => ({ ...prev, [activeTab]: e.target.value }))}
        rows={14}
        style={{
          width: "100%",
          fontFamily: "monospace",
          fontSize: 13,
          background: "#111",
          color: "#e0e0e0",
          border: "1px solid #333",
          borderRadius: 6,
          padding: 12,
          boxSizing: "border-box",
        }}
        placeholder={ENDPOINTS[activeTab].method === "GET" ? "// GET request — no body needed" : "// JSON payload"}
      />

      <button
        onClick={send}
        disabled={loading}
        style={{
          marginTop: 8,
          padding: "10px 24px",
          background: loading ? "#333" : "#a78bfa",
          color: loading ? "#666" : "#000",
          border: "none",
          borderRadius: 6,
          cursor: loading ? "not-allowed" : "pointer",
          fontFamily: "monospace",
          fontWeight: 700,
          fontSize: 14,
        }}
      >
        {loading ? "Sending..." : "Send"}
      </button>

      <pre
        style={{
          background: "#111",
          color: "#0f0",
          padding: 16,
          marginTop: 16,
          minHeight: 200,
          overflow: "auto",
          borderRadius: 6,
          border: "1px solid #1a3a1a",
          fontSize: 13,
        }}
      >
        {response || "// Response will appear here"}
      </pre>
    </div>
  );
}
