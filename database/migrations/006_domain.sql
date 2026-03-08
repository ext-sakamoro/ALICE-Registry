-- ALICE Registry: Domain-specific tables
-- SDF model registry, versioning, search

CREATE TABLE IF NOT EXISTS repositories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth.users(id),
    name TEXT NOT NULL,
    description TEXT,
    visibility TEXT NOT NULL DEFAULT 'public' CHECK (visibility IN ('public','private','unlisted')),
    total_versions INTEGER DEFAULT 0,
    total_downloads BIGINT DEFAULT 0,
    total_size_bytes BIGINT DEFAULT 0,
    license TEXT DEFAULT 'MIT',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, name)
);
CREATE INDEX idx_repo_project ON repositories(project_id);
CREATE INDEX idx_repo_user ON repositories(user_id);
CREATE INDEX idx_repo_name ON repositories(name);

CREATE TABLE IF NOT EXISTS model_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth.users(id),
    tag TEXT NOT NULL DEFAULT 'latest',
    version TEXT NOT NULL,
    format TEXT NOT NULL CHECK (format IN ('sdf-binary','sdf-text','obj','gltf','usd','step','stl','ply')),
    size_bytes BIGINT NOT NULL DEFAULT 0,
    compressed_bytes BIGINT DEFAULT 0,
    sha256 TEXT NOT NULL,
    node_count INTEGER DEFAULT 0,
    bounding_box DOUBLE PRECISION[6],
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (repo_id, tag)
);
CREATE INDEX idx_version_repo ON model_versions(repo_id);
CREATE INDEX idx_version_tag ON model_versions(tag);

CREATE TABLE IF NOT EXISTS model_diffs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth.users(id),
    version_a TEXT NOT NULL,
    version_b TEXT NOT NULL,
    added_nodes INTEGER DEFAULT 0,
    removed_nodes INTEGER DEFAULT 0,
    modified_nodes INTEGER DEFAULT 0,
    diff_size_bytes BIGINT DEFAULT 0,
    similarity_pct DOUBLE PRECISION DEFAULT 100.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_diff_repo ON model_diffs(repo_id);

-- Row Level Security
ALTER TABLE repositories ENABLE ROW LEVEL SECURITY;
ALTER TABLE model_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE model_diffs ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users manage own repos" ON repositories FOR ALL USING (auth.uid() = user_id);
CREATE POLICY "Public repos visible to all" ON repositories FOR SELECT USING (visibility = 'public' OR auth.uid() = user_id);
CREATE POLICY "Users manage own versions" ON model_versions FOR ALL USING (auth.uid() = user_id);
CREATE POLICY "Users manage own diffs" ON model_diffs FOR ALL USING (auth.uid() = user_id);
