use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use things3_core::ThingsId;
use tokio::sync::broadcast;
use uuid::Uuid;

use super::types::{Event, EventType};

/// Event filter for subscriptions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventFilter {
    pub event_types: Option<Vec<EventType>>,
    pub entity_ids: Option<Vec<ThingsId>>,
    pub sources: Option<Vec<String>>,
    pub since: Option<DateTime<Utc>>,
}

impl EventFilter {
    /// Check if an event matches this filter
    #[must_use]
    pub fn matches(&self, event: &Event) -> bool {
        // Check event types
        if let Some(ref types) = self.event_types {
            if !types
                .iter()
                .any(|t| std::mem::discriminant(t) == std::mem::discriminant(&event.event_type))
            {
                return false;
            }
        }

        // Check entity IDs (applies to task/project/area events; progress events have no entity ID)
        if let Some(ref ids) = self.entity_ids {
            let event_entity_id: Option<&ThingsId> = match &event.event_type {
                EventType::TaskCreated { task_id }
                | EventType::TaskUpdated { task_id }
                | EventType::TaskDeleted { task_id }
                | EventType::TaskCompleted { task_id }
                | EventType::TaskCancelled { task_id } => Some(task_id),
                EventType::ProjectCreated { project_id }
                | EventType::ProjectUpdated { project_id }
                | EventType::ProjectDeleted { project_id }
                | EventType::ProjectCompleted { project_id } => Some(project_id),
                EventType::AreaCreated { area_id }
                | EventType::AreaUpdated { area_id }
                | EventType::AreaDeleted { area_id } => Some(area_id),
                EventType::ProgressStarted { .. }
                | EventType::ProgressUpdated { .. }
                | EventType::ProgressCompleted { .. }
                | EventType::ProgressFailed { .. } => None,
            };

            if let Some(entity_id) = event_entity_id {
                if !ids.contains(entity_id) {
                    return false;
                }
            }
        }

        // Check sources
        if let Some(ref sources) = self.sources {
            if !sources.contains(&event.source) {
                return false;
            }
        }

        // Check timestamp
        if let Some(since) = self.since {
            if event.timestamp < since {
                return false;
            }
        }

        true
    }
}

/// Event subscription
#[derive(Debug, Clone)]
pub struct EventSubscription {
    pub id: Uuid,
    pub filter: EventFilter,
    pub sender: broadcast::Sender<Event>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Event, EventType};
    use chrono::Utc;
    use things3_core::ThingsId;
    use tokio::sync::broadcast;
    use uuid::Uuid;

    #[test]
    fn test_event_filter_matching() {
        let task_id = ThingsId::new_v4();
        let event = Event {
            id: Uuid::new_v4(),
            event_type: EventType::TaskCreated {
                task_id: task_id.clone(),
            },
            timestamp: Utc::now(),
            data: None,
            source: "test".to_string(),
        };

        let filter = EventFilter {
            event_types: Some(vec![EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            }]),
            entity_ids: None,
            sources: None,
            since: None,
        };

        // Should match event type
        assert!(filter.matches(&event));

        let filter_no_match = EventFilter {
            event_types: Some(vec![EventType::TaskUpdated {
                task_id: ThingsId::new_v4(),
            }]),
            entity_ids: None,
            sources: None,
            since: None,
        };

        // Should not match different event type
        assert!(!filter_no_match.matches(&event));
    }

    #[test]
    fn test_event_filter_entity_ids() {
        let task_id = ThingsId::new_v4();
        let event = Event {
            id: Uuid::new_v4(),
            event_type: EventType::TaskCreated {
                task_id: task_id.clone(),
            },
            timestamp: Utc::now(),
            data: None,
            source: "test".to_string(),
        };

        let filter = EventFilter {
            event_types: None,
            entity_ids: Some(vec![task_id]),
            sources: None,
            since: None,
        };

        assert!(filter.matches(&event));

        let filter_no_match = EventFilter {
            event_types: None,
            entity_ids: Some(vec![ThingsId::new_v4()]),
            sources: None,
            since: None,
        };

        assert!(!filter_no_match.matches(&event));
    }

    #[test]
    fn test_event_filter_sources() {
        let event = Event {
            id: Uuid::new_v4(),
            event_type: EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            },
            timestamp: Utc::now(),
            data: None,
            source: "test_source".to_string(),
        };

        let filter = EventFilter {
            event_types: None,
            entity_ids: None,
            sources: Some(vec!["test_source".to_string()]),
            since: None,
        };

        assert!(filter.matches(&event));

        let filter_no_match = EventFilter {
            event_types: None,
            entity_ids: None,
            sources: Some(vec!["other_source".to_string()]),
            since: None,
        };

        assert!(!filter_no_match.matches(&event));
    }

    #[test]
    fn test_event_filter_timestamp() {
        let now = Utc::now();
        let past = now - chrono::Duration::hours(1);
        let future = now + chrono::Duration::hours(1);

        let event = Event {
            id: Uuid::new_v4(),
            event_type: EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            },
            timestamp: now,
            data: None,
            source: "test".to_string(),
        };

        let filter = EventFilter {
            event_types: None,
            entity_ids: None,
            sources: None,
            since: Some(past),
        };

        assert!(filter.matches(&event));

        let filter_no_match = EventFilter {
            event_types: None,
            entity_ids: None,
            sources: None,
            since: Some(future),
        };

        assert!(!filter_no_match.matches(&event));
    }

    #[test]
    fn test_event_filter_all_event_types() {
        let task_id = ThingsId::new_v4();
        let project_id = ThingsId::new_v4();
        let area_id = ThingsId::new_v4();
        let operation_id = Uuid::new_v4();

        let events = vec![
            Event {
                id: Uuid::new_v4(),
                event_type: EventType::TaskCreated {
                    task_id: task_id.clone(),
                },
                timestamp: Utc::now(),
                data: None,
                source: "test".to_string(),
            },
            Event {
                id: Uuid::new_v4(),
                event_type: EventType::ProjectCreated {
                    project_id: project_id.clone(),
                },
                timestamp: Utc::now(),
                data: None,
                source: "test".to_string(),
            },
            Event {
                id: Uuid::new_v4(),
                event_type: EventType::AreaCreated {
                    area_id: area_id.clone(),
                },
                timestamp: Utc::now(),
                data: None,
                source: "test".to_string(),
            },
            Event {
                id: Uuid::new_v4(),
                event_type: EventType::ProgressStarted { operation_id },
                timestamp: Utc::now(),
                data: None,
                source: "test".to_string(),
            },
        ];

        for event in events {
            let filter = EventFilter {
                event_types: None,
                entity_ids: None,
                sources: None,
                since: None,
            };
            assert!(filter.matches(&event));
        }
    }

    #[test]
    fn test_event_filter_entity_id_extraction() {
        let task_id = ThingsId::new_v4();
        let project_id = ThingsId::new_v4();
        let area_id = ThingsId::new_v4();
        let operation_id = Uuid::new_v4();

        let events: Vec<(EventType, Option<ThingsId>)> = vec![
            (
                EventType::TaskCreated {
                    task_id: task_id.clone(),
                },
                Some(task_id.clone()),
            ),
            (
                EventType::TaskUpdated {
                    task_id: task_id.clone(),
                },
                Some(task_id.clone()),
            ),
            (
                EventType::TaskDeleted {
                    task_id: task_id.clone(),
                },
                Some(task_id.clone()),
            ),
            (
                EventType::TaskCompleted {
                    task_id: task_id.clone(),
                },
                Some(task_id.clone()),
            ),
            (
                EventType::TaskCancelled {
                    task_id: task_id.clone(),
                },
                Some(task_id.clone()),
            ),
            (
                EventType::ProjectCreated {
                    project_id: project_id.clone(),
                },
                Some(project_id.clone()),
            ),
            (
                EventType::ProjectUpdated {
                    project_id: project_id.clone(),
                },
                Some(project_id.clone()),
            ),
            (
                EventType::ProjectDeleted {
                    project_id: project_id.clone(),
                },
                Some(project_id.clone()),
            ),
            (
                EventType::ProjectCompleted {
                    project_id: project_id.clone(),
                },
                Some(project_id.clone()),
            ),
            (
                EventType::AreaCreated {
                    area_id: area_id.clone(),
                },
                Some(area_id.clone()),
            ),
            (
                EventType::AreaUpdated {
                    area_id: area_id.clone(),
                },
                Some(area_id.clone()),
            ),
            (
                EventType::AreaDeleted {
                    area_id: area_id.clone(),
                },
                Some(area_id.clone()),
            ),
            (EventType::ProgressStarted { operation_id }, None),
            (EventType::ProgressUpdated { operation_id }, None),
            (EventType::ProgressCompleted { operation_id }, None),
            (EventType::ProgressFailed { operation_id }, None),
        ];

        for (event_type, expected_id) in events {
            let event = Event {
                id: Uuid::new_v4(),
                event_type,
                timestamp: Utc::now(),
                data: None,
                source: "test".to_string(),
            };

            let filter = EventFilter {
                event_types: None,
                entity_ids: expected_id.map(|id| vec![id]),
                sources: None,
                since: None,
            };

            assert!(filter.matches(&event));
        }
    }

    #[test]
    fn test_event_filter_creation() {
        let filter = EventFilter {
            event_types: Some(vec![EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            }]),
            entity_ids: Some(vec![ThingsId::new_v4()]),
            sources: Some(vec!["test".to_string()]),
            since: Some(Utc::now()),
        };

        assert!(filter.event_types.is_some());
        assert!(filter.entity_ids.is_some());
        assert!(filter.sources.is_some());
        assert!(filter.since.is_some());
    }

    #[test]
    fn test_event_filter_serialization() {
        let filter = EventFilter {
            event_types: Some(vec![EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            }]),
            entity_ids: Some(vec![ThingsId::new_v4()]),
            sources: Some(vec!["test".to_string()]),
            since: Some(Utc::now()),
        };

        let json = serde_json::to_string(&filter).unwrap();
        let deserialized: EventFilter = serde_json::from_str(&json).unwrap();

        assert_eq!(filter.event_types, deserialized.event_types);
        assert_eq!(filter.entity_ids, deserialized.entity_ids);
        assert_eq!(filter.sources, deserialized.sources);
    }

    #[tokio::test]
    async fn test_event_filter_serialization_roundtrip() {
        let original_filter = EventFilter {
            event_types: Some(vec![
                EventType::TaskCreated {
                    task_id: ThingsId::new_v4(),
                },
                EventType::ProjectCreated {
                    project_id: ThingsId::new_v4(),
                },
            ]),
            entity_ids: Some(vec![ThingsId::new_v4(), ThingsId::new_v4()]),
            sources: Some(vec![
                "test_source".to_string(),
                "another_source".to_string(),
            ]),
            since: Some(Utc::now()),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&original_filter).unwrap();

        // Deserialize back to EventFilter
        let deserialized_filter: EventFilter = serde_json::from_str(&json).unwrap();

        assert_eq!(original_filter.event_types, deserialized_filter.event_types);
        assert_eq!(original_filter.entity_ids, deserialized_filter.entity_ids);
        assert_eq!(original_filter.sources, deserialized_filter.sources);
        assert_eq!(original_filter.since, deserialized_filter.since);
    }

    #[tokio::test]
    async fn test_event_filter_matching_with_timestamp() {
        let filter = EventFilter {
            event_types: Some(vec![EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            }]),
            entity_ids: None,
            sources: None,
            since: Some(Utc::now() - chrono::Duration::hours(1)),
        };

        let event = Event {
            event_type: EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            },
            id: Uuid::new_v4(),
            source: "test".to_string(),
            timestamp: Utc::now(),
            data: None,
        };

        assert!(filter.matches(&event));
    }

    #[tokio::test]
    async fn test_event_filter_matching_with_sources() {
        let filter = EventFilter {
            event_types: None,
            entity_ids: None,
            sources: Some(vec!["test_source".to_string()]),
            since: None,
        };

        let event = Event {
            event_type: EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            },
            id: Uuid::new_v4(),
            source: "test_source".to_string(),
            timestamp: Utc::now(),
            data: None,
        };

        assert!(filter.matches(&event));
    }

    #[tokio::test]
    async fn test_event_filter_matching_with_entity_ids() {
        let entity_id = ThingsId::new_v4();
        let filter = EventFilter {
            event_types: None,
            entity_ids: Some(vec![entity_id.clone()]),
            sources: None,
            since: None,
        };

        let event = Event {
            event_type: EventType::TaskCreated { task_id: entity_id },
            id: Uuid::new_v4(),
            source: "test".to_string(),
            timestamp: Utc::now(),
            data: None,
        };

        assert!(filter.matches(&event));
    }

    #[tokio::test]
    async fn test_event_filter_matching_no_match() {
        let filter = EventFilter {
            event_types: Some(vec![EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            }]),
            entity_ids: None,
            sources: None,
            since: None,
        };

        let event = Event {
            event_type: EventType::ProjectCreated {
                project_id: ThingsId::new_v4(),
            },
            id: Uuid::new_v4(),
            source: "test".to_string(),
            timestamp: Utc::now(),
            data: None,
        };

        assert!(!filter.matches(&event));
    }

    #[tokio::test]
    async fn test_event_filter_matching_empty_filter() {
        let filter = EventFilter {
            event_types: None,
            entity_ids: None,
            sources: None,
            since: None,
        };

        let event = Event {
            event_type: EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            },
            id: Uuid::new_v4(),
            source: "test".to_string(),
            timestamp: Utc::now(),
            data: None,
        };

        // Empty filter should match all events
        assert!(filter.matches(&event));
    }

    #[test]
    fn test_event_subscription_creation() {
        let subscription_id = Uuid::new_v4();
        let filter = EventFilter {
            event_types: None,
            entity_ids: None,
            sources: None,
            since: None,
        };
        let (sender, _receiver) = broadcast::channel(100);

        let subscription = EventSubscription {
            id: subscription_id,
            filter,
            sender,
        };

        assert_eq!(subscription.id, subscription_id);
    }
}
