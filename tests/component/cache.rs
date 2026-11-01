//! Real Valkey: the cache client connects, stores and reads back.

use rust_template::infrastructure::cache::CacheClient;

use crate::cache_config;
use crate::common::unique;

#[tokio::test]
async fn cache_round_trips_against_real_valkey() {
    let mut cache = CacheClient::connect(&cache_config())
        .await
        .expect("the component cache must be reachable");
    assert!(cache.is_initialized(), "the client must have connected");

    let key = format!("component:{}", unique("cache"));
    cache.set(&key, "value-1").await.expect("SET");
    assert_eq!(
        cache.get(&key).await.expect("GET"),
        Some("value-1".to_owned())
    );

    assert_eq!(cache.del(&key).await.expect("DEL"), 1);
    assert_eq!(cache.get(&key).await.expect("GET after DEL"), None);

    cache.close();
}
