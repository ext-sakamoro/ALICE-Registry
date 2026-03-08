use api_gateway::TokenBucket;

#[test]
fn test_token_bucket_full_drain() {
    let mut bucket = TokenBucket::new(5.0, 0.0);
    let mut count = 0;
    while bucket.try_consume() {
        count += 1;
    }
    assert_eq!(count, 5);
}

#[test]
fn test_token_bucket_partial_consume() {
    let mut bucket = TokenBucket::new(10.0, 0.0);
    assert!(bucket.try_consume_tokens(7.5));
    assert!(bucket.try_consume_tokens(2.5));
    assert!(!bucket.try_consume_tokens(0.01));
}

#[test]
fn test_token_bucket_remaining_after_operations() {
    let mut bucket = TokenBucket::new(100.0, 0.0);
    let _ = bucket.try_consume_tokens(30.0);
    let _ = bucket.try_consume_tokens(20.0);
    assert!((bucket.remaining() - 50.0).abs() < f64::EPSILON);
}
