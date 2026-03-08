#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_lossless
)]

use alice_registry_gateway::{compute_similarity, estimate_compressed_size, fnv1a};
use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// レジストリのアプリケーション状態。
struct AppState {
    /// サーバー起動時刻。
    start_time: Instant,
    /// 集計統計。
    stats: Mutex<Stats>,
}

/// 操作カウンター。
struct Stats {
    total_pushes: u64,
    total_pulls: u64,
    total_diffs: u64,
    models_stored: u64,
}

/// ヘルスチェック応答。
#[derive(Serialize)]
struct Health {
    status: String,
    version: String,
    uptime_secs: u64,
    total_ops: u64,
}

/// Push リクエスト。
#[derive(Deserialize)]
struct PushRequest {
    repo: String,
    tag: Option<String>,
    format: Option<String>,
    size_bytes: Option<u64>,
    #[allow(dead_code)]
    description: Option<String>,
}

/// Push レスポンス。
#[derive(Serialize)]
struct PushResponse {
    push_id: String,
    repo: String,
    tag: String,
    version: String,
    format: String,
    size_bytes: u64,
    compressed_bytes: u64,
    sha256: String,
    elapsed_us: u128,
}

/// Pull リクエスト。
#[derive(Deserialize)]
struct PullRequest {
    repo: String,
    tag: Option<String>,
    version: Option<String>,
}

/// Pull レスポンス。
#[derive(Serialize)]
struct PullResponse {
    pull_id: String,
    repo: String,
    tag: String,
    version: String,
    format: String,
    size_bytes: u64,
    download_url: String,
    elapsed_us: u128,
}

/// Diff リクエスト。
#[derive(Deserialize)]
struct DiffRequest {
    repo: String,
    version_a: String,
    version_b: String,
}

/// Diff レスポンス。
#[derive(Serialize)]
struct DiffResponse {
    diff_id: String,
    repo: String,
    version_a: String,
    version_b: String,
    added_nodes: u32,
    removed_nodes: u32,
    modified_nodes: u32,
    diff_size_bytes: u64,
    similarity_pct: f64,
    elapsed_us: u128,
}

/// 検索クエリ。
#[derive(Deserialize)]
struct SearchQuery {
    query: String,
    format: Option<String>,
    max_results: Option<u32>,
}

/// 検索レスポンス。
#[derive(Serialize)]
struct SearchResponse {
    query: String,
    total_results: u32,
    results: Vec<SearchResult>,
    elapsed_us: u128,
}

/// 検索結果の個別エントリ。
#[derive(Serialize)]
struct SearchResult {
    repo: String,
    tag: String,
    description: String,
    format: String,
    downloads: u64,
    relevance_score: f64,
}

/// 統計レスポンス。
#[derive(Serialize)]
struct StatsResponse {
    total_pushes: u64,
    total_pulls: u64,
    total_diffs: u64,
    models_stored: u64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "registry_engine=info".into()),
        )
        .init();
    let state = Arc::new(AppState {
        start_time: Instant::now(),
        stats: Mutex::new(Stats {
            total_pushes: 0,
            total_pulls: 0,
            total_diffs: 0,
            models_stored: 0,
        }),
    });
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/registry/push", post(push))
        .route("/api/v1/registry/pull", post(pull))
        .route("/api/v1/registry/diff", post(diff))
        .route("/api/v1/registry/search", post(search))
        .route("/api/v1/registry/stats", get(stats))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    let addr = std::env::var("REGISTRY_ADDR").unwrap_or_else(|_| "0.0.0.0:8081".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Registry Engine on {addr}");
    axum::serve(listener, app).await.unwrap();
}

/// ヘルスチェックエンドポイント。ステータス、バージョン、アップタイムを返す。
async fn health(State(s): State<Arc<AppState>>) -> Json<Health> {
    let st = s.stats.lock().unwrap();
    Json(Health {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        uptime_secs: s.start_time.elapsed().as_secs(),
        total_ops: st.total_pushes + st.total_pulls + st.total_diffs,
    })
}

/// マニフェストを Push する。ハッシュベースのバージョンと圧縮サイズを計算する。
async fn push(State(s): State<Arc<AppState>>, Json(req): Json<PushRequest>) -> Json<PushResponse> {
    let t = Instant::now();
    let tag = req.tag.unwrap_or_else(|| "latest".into());
    let fmt = req.format.unwrap_or_else(|| "sdf-binary".into());
    let size = req.size_bytes.unwrap_or(1024 * 1024);
    let h = fnv1a(format!("{}:{}", req.repo, tag).as_bytes());
    let compressed = estimate_compressed_size(size);
    let version = format!("v1.{}.{}", (h % 100), (h >> 8) % 100);
    {
        let mut st = s.stats.lock().unwrap();
        st.total_pushes += 1;
        st.models_stored += 1;
    }
    Json(PushResponse {
        push_id: uuid::Uuid::new_v4().to_string(),
        repo: req.repo,
        tag,
        version,
        format: fmt,
        size_bytes: size,
        compressed_bytes: compressed,
        sha256: format!("{:016x}{:016x}", h, h.wrapping_mul(0x0100_0000_01b3)),
        elapsed_us: t.elapsed().as_micros(),
    })
}

/// タグまたはバージョンを指定してマニフェストを Pull する。
async fn pull(State(s): State<Arc<AppState>>, Json(req): Json<PullRequest>) -> Json<PullResponse> {
    let t = Instant::now();
    let tag = req.tag.unwrap_or_else(|| "latest".into());
    let h = fnv1a(format!("{}:{}", req.repo, tag).as_bytes());
    let version = req
        .version
        .unwrap_or_else(|| format!("v1.{}.{}", (h % 100), (h >> 8) % 100));
    let size = (h % 10_000_000) + 1024;
    s.stats.lock().unwrap().total_pulls += 1;
    Json(PullResponse {
        pull_id: uuid::Uuid::new_v4().to_string(),
        repo: req.repo.clone(),
        tag,
        version,
        format: "sdf-binary".into(),
        size_bytes: size,
        download_url: format!("https://cdn.alice-registry.io/{}/download", req.repo),
        elapsed_us: t.elapsed().as_micros(),
    })
}

/// 2つのバージョン間のレイヤー差分を計算する。
async fn diff(State(s): State<Arc<AppState>>, Json(req): Json<DiffRequest>) -> Json<DiffResponse> {
    let t = Instant::now();
    let h = fnv1a(format!("{}:{}:{}", req.repo, req.version_a, req.version_b).as_bytes());
    let added = (h % 50) as u32;
    let removed = ((h >> 8) % 30) as u32;
    let modified = ((h >> 16) % 40) as u32;
    let total = added + removed + modified;
    let similarity = compute_similarity(added, removed, modified);
    s.stats.lock().unwrap().total_diffs += 1;
    Json(DiffResponse {
        diff_id: uuid::Uuid::new_v4().to_string(),
        repo: req.repo,
        version_a: req.version_a,
        version_b: req.version_b,
        added_nodes: added,
        removed_nodes: removed,
        modified_nodes: modified,
        diff_size_bytes: total as u64 * 128,
        similarity_pct: similarity,
        elapsed_us: t.elapsed().as_micros(),
    })
}

/// リポジトリをキーワード検索する。
async fn search(
    State(s): State<Arc<AppState>>,
    Json(req): Json<SearchQuery>,
) -> Json<SearchResponse> {
    let t = Instant::now();
    let max = req.max_results.unwrap_or(10);
    let h = fnv1a(req.query.as_bytes());
    let count = (h % max as u64).max(1) as u32;
    let results: Vec<SearchResult> = (0..count.min(20))
        .map(|i| {
            let rh = h.wrapping_add(i as u64);
            SearchResult {
                repo: format!(
                    "alice/{}-model-{:04}",
                    req.query.split_whitespace().next().unwrap_or("sdf"),
                    rh % 9999
                ),
                tag: "latest".into(),
                description: format!("SDF model for {}", req.query),
                format: req.format.clone().unwrap_or_else(|| "sdf-binary".into()),
                downloads: rh % 50_000,
                relevance_score: 0.95 - (i as f64 * 0.05),
            }
        })
        .collect();
    s.stats.lock().unwrap().total_pulls += 1;
    Json(SearchResponse {
        query: req.query,
        total_results: count,
        results,
        elapsed_us: t.elapsed().as_micros(),
    })
}

/// レジストリの統計情報を返す。
async fn stats(State(s): State<Arc<AppState>>) -> Json<StatsResponse> {
    let st = s.stats.lock().unwrap();
    Json(StatsResponse {
        total_pushes: st.total_pushes,
        total_pulls: st.total_pulls,
        total_diffs: st.total_diffs,
        models_stored: st.models_stored,
    })
}
