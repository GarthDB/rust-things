use std::sync::Arc;
use things3_core::ThingsId;
use tokio::sync::broadcast;

use super::broadcaster::EventBroadcaster;
use super::filter::EventFilter;
use super::types::{Event, EventType};

/// Event listener for handling events
pub struct EventListener {
    broadcaster: Arc<EventBroadcaster>,
}

impl EventListener {
    /// Create a new event listener
    #[must_use]
    pub fn new(broadcaster: Arc<EventBroadcaster>) -> Self {
        Self { broadcaster }
    }

    /// Subscribe to specific event types
    pub async fn subscribe_to_events(
        &mut self,
        event_types: Vec<EventType>,
    ) -> broadcast::Receiver<Event> {
        let filter = EventFilter {
            event_types: Some(event_types),
            entity_ids: None,
            sources: None,
            since: None,
        };

        self.broadcaster.subscribe(filter).await
    }

    /// Subscribe to events for a specific entity
    pub async fn subscribe_to_entity(&mut self, entity_id: ThingsId) -> broadcast::Receiver<Event> {
        let filter = EventFilter {
            event_types: None,
            entity_ids: Some(vec![entity_id]),
            sources: None,
            since: None,
        };

        self.broadcaster.subscribe(filter).await
    }

    /// Subscribe to all events
    #[must_use]
    pub fn subscribe_to_all(&self) -> broadcast::Receiver<Event> {
        self.broadcaster.subscribe_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventBroadcaster, EventType};
    use std::sync::Arc;
    use things3_core::ThingsId;

    #[tokio::test]
    async fn test_event_listener_creation() {
        let broadcaster = EventBroadcaster::new();
        // Verify EventListener::new succeeds without panicking
        let _listener = EventListener::new(Arc::new(broadcaster));
    }

    #[tokio::test]
    async fn test_event_listener_subscribe_to_events() {
        let broadcaster = EventBroadcaster::new();
        let mut listener = EventListener::new(Arc::new(broadcaster));

        let event_types = vec![EventType::TaskCreated {
            task_id: ThingsId::new_v4(),
        }];
        let mut receiver = listener.subscribe_to_events(event_types).await;

        // This should not panic
        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_event_listener_subscribe_to_entity() {
        let broadcaster = EventBroadcaster::new();
        let mut listener = EventListener::new(Arc::new(broadcaster));

        let entity_id = ThingsId::new_v4();
        let mut receiver = listener.subscribe_to_entity(entity_id).await;

        // This should not panic
        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_event_listener_subscribe_to_all() {
        let broadcaster = EventBroadcaster::new();
        let listener = EventListener::new(Arc::new(broadcaster));

        let mut receiver = listener.subscribe_to_all();

        // This should not panic
        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_event_listener_with_actual_broadcaster() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let mut listener = EventListener::new(broadcaster);

        let event_types = vec![EventType::TaskCreated {
            task_id: ThingsId::new_v4(),
        }];
        let mut receiver = listener.subscribe_to_events(event_types).await;

        // This should not panic
        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_event_listener_subscribe_to_entity_with_actual_broadcaster() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let mut listener = EventListener::new(broadcaster);

        let entity_id = ThingsId::new_v4();
        let mut receiver = listener.subscribe_to_entity(entity_id).await;

        // This should not panic
        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_event_listener_subscribe_to_all_with_actual_broadcaster() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let listener = EventListener::new(broadcaster);

        let mut receiver = listener.subscribe_to_all();

        // This should not panic
        assert!(receiver.try_recv().is_err());
    }
}
