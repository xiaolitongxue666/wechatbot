//! Backend integration tests: event queue.

mod common;

use wechatbot::queue::{EventQueue, InMemoryEventQueue};
use std::sync::Arc;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn in_memory_queue_multi_event() {
    let queue = InMemoryEventQueue::with_capacity(10);
    for i in 0..5 {
        queue
            .publish(format!("event-{i}"))
            .await
            .unwrap();
    }

    for i in 0..5 {
        let payload = queue.consume().await.unwrap();
        assert_eq!(payload, format!("event-{i}"), "events should be FIFO");
    }
}

#[tokio::test]
async fn in_memory_queue_consume_blocks() {
    let queue = Arc::new(InMemoryEventQueue::with_capacity(4));
    let q = queue.clone();

    let handle = tokio::spawn(async move { q.consume().await });

    let result = timeout(Duration::from_millis(200), handle).await;
    assert!(
        result.is_err() || matches!(result, Ok(Err(_))),
        "consume should block until a message is published"
    );

    queue.publish("delayed-event".to_string()).await.unwrap();
    // The spawned task should now be able to consume
}

#[tokio::test]
async fn in_memory_queue_publish_consume_ordering() {
    let queue = InMemoryEventQueue::with_capacity(8);
    let messages = vec!["first", "second", "third", "fourth"];

    for msg in &messages {
        queue.publish(msg.to_string()).await.unwrap();
    }

    for expected in &messages {
        let received = queue.consume().await.unwrap();
        assert_eq!(received, *expected);
    }
}
