export default function LandingPage() {
  const features = [
    {
      title: "OCI-Compatible Push/Pull",
      description:
        "Standard OCI Image Manifest v1 push and pull with chunked upload, resumable transfers, and multi-arch manifests.",
    },
    {
      title: "Layer Diffing",
      description:
        "Content-addressable layer diff between any two tags. Identify added, modified, and removed layers with size deltas.",
    },
    {
      title: "Content-Addressable Search",
      description:
        "Full-text and label-based search across all repositories. Filter by namespace, digest, push time, and custom labels.",
    },
    {
      title: "Garbage Collection",
      description:
        "Automatic GC for unreferenced blobs and dangling manifests. Policy-driven retention with dry-run preview.",
    },
    {
      title: "Access Control",
      description:
        "Per-repository RBAC with push, pull, and admin roles. Token-based auth compatible with Docker CLI and Helm.",
    },
    {
      title: "Multi-Format Support",
      description:
        "Stores OCI images, Helm charts, WASM modules, and generic artifacts under a unified content-addressable store.",
    },
  ];

  return (
    <div
      style={{
        minHeight: "100vh",
        background: "linear-gradient(135deg, #0a0a0a, #13001a)",
        color: "#fff",
        fontFamily: "system-ui, sans-serif",
      }}
    >
      <header
        style={{
          padding: "24px 48px",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          borderBottom: "1px solid #ffffff10",
        }}
      >
        <h2 style={{ margin: 0, color: "#a78bfa" }}>ALICE Registry</h2>
        <a href="/dashboard/console" style={{ color: "#a78bfa", textDecoration: "none", fontWeight: 600 }}>
          Console →
        </a>
      </header>

      <main style={{ maxWidth: 960, margin: "0 auto", padding: "80px 24px", textAlign: "center" }}>
        <div
          style={{
            display: "inline-block",
            background: "#a78bfa20",
            color: "#a78bfa",
            borderRadius: 20,
            padding: "4px 16px",
            fontSize: 12,
            fontWeight: 600,
            marginBottom: 24,
            letterSpacing: 1,
          }}
        >
          CONTAINER AND ARTIFACT REGISTRY SAAS
        </div>

        <h1 style={{ fontSize: 48, marginBottom: 16, lineHeight: 1.1 }}>
          Push, Pull, Diff
          <br />
          <span style={{ color: "#a78bfa" }}>Any Artifact, Anywhere</span>
        </h1>

        <p style={{ fontSize: 20, color: "#aaa", marginBottom: 48, maxWidth: 600, margin: "0 auto 48px" }}>
          OCI-compatible container and artifact registry with layer diffing and content-addressable search — built for production at scale.
        </p>

        <div style={{ display: "flex", gap: 16, justifyContent: "center", marginBottom: 80 }}>
          <a
            href="/dashboard/console"
            style={{
              background: "#a78bfa",
              color: "#000",
              padding: "14px 32px",
              borderRadius: 8,
              textDecoration: "none",
              fontWeight: 700,
            }}
          >
            Open Console
          </a>
          <a
            href="#features"
            style={{
              background: "#ffffff10",
              color: "#fff",
              padding: "14px 32px",
              borderRadius: 8,
              textDecoration: "none",
              fontWeight: 600,
            }}
          >
            Learn More
          </a>
        </div>

        <div
          id="features"
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(3, 1fr)",
            gap: 24,
            textAlign: "left",
          }}
        >
          {features.map((f) => (
            <div
              key={f.title}
              style={{
                background: "#ffffff08",
                borderRadius: 12,
                padding: 24,
                border: "1px solid #ffffff10",
              }}
            >
              <h3 style={{ margin: "0 0 12px", color: "#a78bfa", fontSize: 16 }}>{f.title}</h3>
              <p style={{ color: "#aaa", margin: 0, lineHeight: 1.6, fontSize: 14 }}>{f.description}</p>
            </div>
          ))}
        </div>
      </main>

      <footer style={{ textAlign: "center", padding: "32px", borderTop: "1px solid #ffffff10", color: "#444", fontSize: 12 }}>
        ALICE Registry · AGPL-3.0-or-later · Project A.L.I.C.E.
      </footer>
    </div>
  );
}
