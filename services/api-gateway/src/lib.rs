//! ALICE Registry API Gateway ライブラリ。
//!
//! JWT/API-Key 認証、トークンバケットレートリミット、リバースプロキシを提供する。
//!
//! # Example
//!
//! ```
//! use api_gateway::TokenBucket;
//!
//! let mut bucket = TokenBucket::new(10.0, 1.0);
//! assert!(bucket.try_consume_tokens(1.0));
//! ```

use std::time::Instant;

/// トークンバケットによるレートリミッター。
///
/// 一定速度でトークンを補充し、リクエスト毎にトークンを消費する。
/// トークンが不足するとリクエストを拒否する。
pub struct TokenBucket {
    /// 現在のトークン残量。
    pub tokens: f64,
    /// バケットの最大容量。
    pub max_tokens: f64,
    /// 1秒あたりのトークン補充速度。
    pub refill_rate: f64,
    /// 最後にトークンを補充した時刻。
    pub last_refill: Instant,
}

impl TokenBucket {
    /// 新しいトークンバケットを作成する。
    ///
    /// `max` は最大トークン数、`rate` は1秒あたりの補充速度。
    #[must_use]
    pub fn new(max: f64, rate: f64) -> Self {
        Self {
            tokens: max,
            max_tokens: max,
            refill_rate: rate,
            last_refill: Instant::now(),
        }
    }

    /// トークンを1つ消費し、成功なら `true` を返す。
    ///
    /// 経過時間に応じてトークンを補充した後、1トークン消費を試みる。
    #[must_use]
    pub fn try_consume(&mut self) -> bool {
        self.try_consume_tokens(1.0)
    }

    /// 指定量のトークンを消費し、成功なら `true` を返す。
    ///
    /// 経過時間に応じてトークンを補充した後、`amount` トークンの消費を試みる。
    #[must_use]
    pub fn try_consume_tokens(&mut self, amount: f64) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
        if self.tokens >= amount {
            self.tokens -= amount;
            true
        } else {
            false
        }
    }

    /// 現在のトークン残量を返す。
    #[must_use]
    pub fn remaining(&self) -> f64 {
        self.tokens
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------
    // 基本テスト
    // -------------------------------------------------------------------

    #[test]
    fn test_new_bucket_full() {
        let bucket = TokenBucket::new(10.0, 1.0);
        assert!((bucket.tokens - 10.0).abs() < f64::EPSILON);
        assert!((bucket.max_tokens - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_consume_reduces_tokens() {
        let mut bucket = TokenBucket::new(10.0, 0.0);
        assert!(bucket.try_consume());
        assert!(bucket.remaining() < 10.0);
    }

    #[test]
    fn test_consume_when_empty_fails() {
        let mut bucket = TokenBucket::new(1.0, 0.0);
        assert!(bucket.try_consume()); // 1 → 0
        assert!(!bucket.try_consume()); // 0 → 不足
    }

    #[test]
    fn test_consume_tokens_exact_amount() {
        let mut bucket = TokenBucket::new(5.0, 0.0);
        assert!(bucket.try_consume_tokens(5.0));
        assert!(!bucket.try_consume_tokens(0.01));
    }

    // -------------------------------------------------------------------
    // 境界値テスト
    // -------------------------------------------------------------------

    #[test]
    fn test_zero_max_tokens() {
        let mut bucket = TokenBucket::new(0.0, 0.0);
        assert!(!bucket.try_consume());
    }

    #[test]
    fn test_zero_refill_rate() {
        let mut bucket = TokenBucket::new(3.0, 0.0);
        assert!(bucket.try_consume());
        assert!(bucket.try_consume());
        assert!(bucket.try_consume());
        assert!(!bucket.try_consume());
    }

    #[test]
    fn test_large_max_tokens() {
        let mut bucket = TokenBucket::new(1_000_000.0, 0.0);
        for _ in 0..1000 {
            assert!(bucket.try_consume());
        }
    }

    #[test]
    fn test_consume_zero_tokens_always_succeeds() {
        let mut bucket = TokenBucket::new(0.0, 0.0);
        assert!(bucket.try_consume_tokens(0.0));
    }

    #[test]
    fn test_remaining_reflects_consumption() {
        let mut bucket = TokenBucket::new(5.0, 0.0);
        let _ = bucket.try_consume_tokens(3.0);
        assert!((bucket.remaining() - 2.0).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------
    // Property-based tests
    // -------------------------------------------------------------------

    use proptest::prelude::*;

    proptest! {
        /// 新規バケットからmax_tokens以下のトークン消費は常に成功する。
        #[test]
        fn prop_consume_within_capacity(max in 1.0f64..1000.0, amount in 0.0f64..1000.0) {
            let mut bucket = TokenBucket::new(max, 0.0);
            if amount <= max {
                prop_assert!(bucket.try_consume_tokens(amount));
            }
        }

        /// 空のバケット（refill_rate=0）からの消費は常に失敗する。
        #[test]
        fn prop_empty_bucket_rejects(amount in 0.01f64..1000.0) {
            let mut bucket = TokenBucket::new(0.0, 0.0);
            prop_assert!(!bucket.try_consume_tokens(amount));
        }

        /// max_tokens を超える消費は不可能。
        #[test]
        fn prop_cannot_consume_more_than_max(max in 0.1f64..100.0, extra in 0.01f64..100.0) {
            let mut bucket = TokenBucket::new(max, 0.0);
            prop_assert!(!bucket.try_consume_tokens(max + extra));
        }
    }
}
