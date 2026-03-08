use alice_registry_gateway::{compute_similarity, estimate_compressed_size, fnv1a};

#[test]
fn test_fnv1a_roundtrip_consistency() {
    // 同一入力は常に同一ハッシュ。
    let inputs = vec![
        b"alice".to_vec(),
        b"registry".to_vec(),
        b"push:v1.0".to_vec(),
        vec![],
    ];
    for input in &inputs {
        assert_eq!(fnv1a(input), fnv1a(input));
    }
}

#[test]
fn test_fnv1a_different_lengths_differ() {
    // 長さの異なる入力はハッシュが異なる。
    assert_ne!(fnv1a(b"a"), fnv1a(b"aa"));
    assert_ne!(fnv1a(b"ab"), fnv1a(b"abc"));
}

#[test]
fn test_estimate_compressed_size_ratio() {
    let original = 1_000_000;
    let compressed = estimate_compressed_size(original);
    // 35% 圧縮率。
    assert_eq!(compressed, 350_000);
}

#[test]
fn test_compute_similarity_range() {
    // 変更なし → 100%
    assert!((compute_similarity(0, 0, 0) - 100.0).abs() < f64::EPSILON);
    // 少量変更 → 高い類似度
    let s = compute_similarity(5, 3, 2);
    assert!(s > 90.0);
    assert!(s <= 100.0);
}

#[test]
fn test_compute_similarity_massive_changes() {
    // 大量変更でも下限は 20%。
    let s = compute_similarity(10000, 10000, 10000);
    assert!((s - 20.0).abs() < f64::EPSILON);
}
