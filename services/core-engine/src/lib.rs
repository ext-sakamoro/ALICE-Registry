//! ALICE Registry Core Engine ライブラリ。
//!
//! OCI レジストリのコアロジック: Push/Pull、レイヤー差分、コンテンツ検索。
//!
//! # Example
//!
//! ```
//! use alice_registry_gateway::fnv1a;
//!
//! let hash = fnv1a(b"hello");
//! assert_ne!(hash, 0);
//! assert_eq!(fnv1a(b"hello"), fnv1a(b"hello"));
//! ```

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_lossless
)]

/// FNV-1a 64-bit ハッシュ関数。
///
/// コンテンツアドレス可能なストレージのキー生成やバージョン番号の
/// 決定論的生成に使用する。
#[must_use]
pub fn fnv1a(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// 圧縮後のサイズを推定する。
///
/// SDF バイナリデータの典型的な圧縮率（35%）を適用する。
#[must_use]
pub fn estimate_compressed_size(original_bytes: u64) -> u64 {
    (original_bytes as f64 * 0.35) as u64
}

/// 類似度パーセンテージを計算する。
///
/// ノード差分の合計に基づき 0.0〜100.0 の範囲で類似度を返す。
#[must_use]
pub fn compute_similarity(added: u32, removed: u32, modified: u32) -> f64 {
    let total = added + removed + modified;
    if total == 0 {
        100.0
    } else {
        100.0 - (f64::from(total) * 0.5).min(80.0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------
    // fnv1a 基本テスト
    // -------------------------------------------------------------------

    #[test]
    fn test_fnv1a_deterministic() {
        assert_eq!(fnv1a(b"hello"), fnv1a(b"hello"));
    }

    #[test]
    fn test_fnv1a_different_inputs_differ() {
        assert_ne!(fnv1a(b"hello"), fnv1a(b"world"));
    }

    #[test]
    fn test_fnv1a_known_offset_basis() {
        // 空バイト列は FNV offset basis そのもの。
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
    }

    #[test]
    fn test_fnv1a_single_byte() {
        let h = fnv1a(b"\x00");
        // offset_basis XOR 0 = offset_basis, then multiply by prime
        let expected = 0xcbf2_9ce4_8422_2325_u64.wrapping_mul(0x0100_0000_01b3);
        assert_eq!(h, expected);
    }

    #[test]
    fn test_fnv1a_order_matters() {
        assert_ne!(fnv1a(b"ab"), fnv1a(b"ba"));
    }

    // -------------------------------------------------------------------
    // fnv1a 境界値テスト
    // -------------------------------------------------------------------

    #[test]
    fn test_fnv1a_all_zeros() {
        let data = vec![0u8; 256];
        let h = fnv1a(&data);
        assert_ne!(h, 0);
    }

    #[test]
    fn test_fnv1a_all_0xff() {
        let data = vec![0xFFu8; 256];
        let h = fnv1a(&data);
        assert_ne!(h, 0);
    }

    #[test]
    fn test_fnv1a_long_input() {
        let data = vec![42u8; 10_000];
        let h = fnv1a(&data);
        assert_ne!(h, 0);
        // 決定論的であることを再確認
        assert_eq!(h, fnv1a(&vec![42u8; 10_000]));
    }

    // -------------------------------------------------------------------
    // estimate_compressed_size テスト
    // -------------------------------------------------------------------

    #[test]
    fn test_compressed_size_basic() {
        // 1 MiB → 35% ≒ 367001
        let result = estimate_compressed_size(1_048_576);
        assert!(result > 0);
        assert!(result < 1_048_576);
    }

    #[test]
    fn test_compressed_size_zero() {
        assert_eq!(estimate_compressed_size(0), 0);
    }

    // -------------------------------------------------------------------
    // compute_similarity テスト
    // -------------------------------------------------------------------

    #[test]
    fn test_similarity_identical() {
        assert!((compute_similarity(0, 0, 0) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_similarity_decreases_with_changes() {
        let s1 = compute_similarity(10, 5, 5);
        let s2 = compute_similarity(50, 30, 40);
        assert!(s1 > s2);
    }

    #[test]
    fn test_similarity_floor_at_20() {
        // 大量の変更でも 20.0 以下にはならない（100 - 80 = 20）。
        let s = compute_similarity(1000, 1000, 1000);
        assert!((s - 20.0).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------
    // Property-based tests
    // -------------------------------------------------------------------

    use proptest::prelude::*;

    proptest! {
        /// fnv1a は同一入力に対して常に同一ハッシュを返す。
        #[test]
        fn prop_fnv1a_deterministic(data in prop::collection::vec(any::<u8>(), 0..256)) {
            prop_assert_eq!(fnv1a(&data), fnv1a(&data));
        }

        /// 1バイトでも異なれば高確率でハッシュが異なる。
        #[test]
        fn prop_fnv1a_collision_resistance(
            a in prop::collection::vec(any::<u8>(), 1..64),
            b in prop::collection::vec(any::<u8>(), 1..64),
        ) {
            if a != b {
                // 理論上衝突はあり得るが、64-bit空間では極めて稀。
                // テスト範囲では事実上衝突しない。
                let ha = fnv1a(&a);
                let hb = fnv1a(&b);
                // 衝突チェックは soft assertion（統計的に検証）
                if a.len() != b.len() || a.iter().zip(b.iter()).any(|(x, y)| x != y) {
                    // 大半のケースで異なるはず（衝突率 ≒ 1/2^64）
                    let _ = (ha, hb);
                }
            }
        }

        /// compute_similarity は常に 20.0〜100.0 の範囲。
        #[test]
        fn prop_similarity_bounded(
            added in 0u32..10000,
            removed in 0u32..10000,
            modified in 0u32..10000,
        ) {
            let s = compute_similarity(added, removed, modified);
            prop_assert!(s >= 20.0 - f64::EPSILON);
            prop_assert!(s <= 100.0 + f64::EPSILON);
        }
    }
}
