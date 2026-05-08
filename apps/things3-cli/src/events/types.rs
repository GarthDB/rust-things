use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use things3_core::ThingsId;
use uuid::Uuid;

/// Event types for Things 3 entities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type")]
pub enum EventType {
    /// Task events
    TaskCreated {
        task_id: ThingsId,
    },
    TaskUpdated {
        task_id: ThingsId,
    },
    TaskDeleted {
        task_id: ThingsId,
    },
    TaskCompleted {
        task_id: ThingsId,
    },
    TaskCancelled {
        task_id: ThingsId,
    },

    /// Project events
    ProjectCreated {
        project_id: ThingsId,
    },
    ProjectUpdated {
        project_id: ThingsId,
    },
    ProjectDeleted {
        project_id: ThingsId,
    },
    ProjectCompleted {
        project_id: ThingsId,
    },

    /// Area events
    AreaCreated {
        area_id: ThingsId,
    },
    AreaUpdated {
        area_id: ThingsId,
    },
    AreaDeleted {
        area_id: ThingsId,
    },

    /// Progress events
    ProgressStarted {
        operation_id: Uuid,
    },
    ProgressUpdated {
        operation_id: Uuid,
    },
    ProgressCompleted {
        operation_id: Uuid,
    },
    ProgressFailed {
        operation_id: Uuid,
    },
}

/// Event data structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: Uuid,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub data: Option<serde_json::Value>,
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use things3_core::ThingsId;
    use uuid::Uuid;

    #[test]
    fn test_event_creation() {
        let event = Event {
            id: Uuid::new_v4(),
            event_type: EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            },
            timestamp: Utc::now(),
            data: None,
            source: "test".to_string(),
        };

        assert!(!event.id.is_nil());
        assert_eq!(event.source, "test");
    }

    #[test]
    fn test_event_creation_with_data() {
        let event = Event {
            id: Uuid::new_v4(),
            event_type: EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            },
            timestamp: Utc::now(),
            data: Some(serde_json::json!({"key": "value"})),
            source: "test".to_string(),
        };

        assert!(!event.id.is_nil());
        assert_eq!(event.source, "test");
        assert!(event.data.is_some());
    }

    #[tokio::test]
    async fn test_event_creation_without_data() {
        let event = Event {
            event_type: EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            },
            id: Uuid::new_v4(),
            source: "test".to_string(),
            timestamp: Utc::now(),
            data: None,
        };

        assert_eq!(event.source, "test");
        assert!(event.data.is_none());
    }

    #[test]
    fn test_all_event_types_creation() {
        let task_id = ThingsId::new_v4();
        let project_id = ThingsId::new_v4();
        let area_id = ThingsId::new_v4();
        let operation_id = Uuid::new_v4();

        // Test all task event types
        let _ = EventType::TaskCreated {
            task_id: task_id.clone(),
        };
        let _ = EventType::TaskUpdated {
            task_id: task_id.clone(),
        };
        let _ = EventType::TaskDeleted {
            task_id: task_id.clone(),
        };
        let _ = EventType::TaskCompleted {
            task_id: task_id.clone(),
        };
        let _ = EventType::TaskCancelled { task_id };

        // Test all project event types
        let _ = EventType::ProjectCreated {
            project_id: project_id.clone(),
        };
        let _ = EventType::ProjectUpdated {
            project_id: project_id.clone(),
        };
        let _ = EventType::ProjectDeleted {
            project_id: project_id.clone(),
        };
        let _ = EventType::ProjectCompleted { project_id };

        // Test all area event types
        let _ = EventType::AreaCreated {
            area_id: area_id.clone(),
        };
        let _ = EventType::AreaUpdated {
            area_id: area_id.clone(),
        };
        let _ = EventType::AreaDeleted { area_id };

        // Test all progress event types
        let _ = EventType::ProgressStarted { operation_id };
        let _ = EventType::ProgressUpdated { operation_id };
        let _ = EventType::ProgressCompleted { operation_id };
        let _ = EventType::ProgressFailed { operation_id };

        // All should compile without errors
    }

    #[tokio::test]
    async fn test_event_type_entity_id_extraction_comprehensive() {
        let task_id = ThingsId::new_v4();
        let project_id = ThingsId::new_v4();
        let area_id = ThingsId::new_v4();
        let operation_id = Uuid::new_v4();

        // Test all event types
        let events = vec![
            EventType::TaskCreated {
                task_id: task_id.clone(),
            },
            EventType::TaskUpdated {
                task_id: task_id.clone(),
            },
            EventType::TaskDeleted {
                task_id: task_id.clone(),
            },
            EventType::TaskCompleted {
                task_id: task_id.clone(),
            },
            EventType::TaskCancelled { task_id },
            EventType::ProjectCreated {
                project_id: project_id.clone(),
            },
            EventType::ProjectUpdated {
                project_id: project_id.clone(),
            },
            EventType::ProjectDeleted {
                project_id: project_id.clone(),
            },
            EventType::ProjectCompleted { project_id },
            EventType::AreaCreated {
                area_id: area_id.clone(),
            },
            EventType::AreaUpdated {
                area_id: area_id.clone(),
            },
            EventType::AreaDeleted { area_id },
            EventType::ProgressStarted { operation_id },
            EventType::ProgressUpdated { operation_id },
            EventType::ProgressCompleted { operation_id },
            EventType::ProgressFailed { operation_id },
        ];

        // Mirror the production event_entity_id match in EventFilter::matches.
        // Entity events yield Some(ThingsId); progress events yield None because
        // operation_id is an internal Uuid, not a ThingsId entity identifier.
        for event_type in &events {
            let extracted_id: Option<&ThingsId> = match event_type {
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

            let is_progress = matches!(
                event_type,
                EventType::ProgressStarted { .. }
                    | EventType::ProgressUpdated { .. }
                    | EventType::ProgressCompleted { .. }
                    | EventType::ProgressFailed { .. }
            );
            if is_progress {
                assert!(
                    extracted_id.is_none(),
                    "progress events must not have a ThingsId"
                );
            } else {
                assert!(
                    extracted_id.is_some(),
                    "entity events must carry a ThingsId"
                );
            }
        }
    }

    #[test]
    fn test_event_serialization() {
        let event = Event {
            id: Uuid::new_v4(),
            event_type: EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            },
            timestamp: Utc::now(),
            data: Some(serde_json::json!({"key": "value"})),
            source: "test".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.source, deserialized.source);
    }

    #[tokio::test]
    async fn test_event_serialization_roundtrip() {
        let original_event = Event {
            event_type: EventType::TaskCreated {
                task_id: ThingsId::new_v4(),
            },
            id: Uuid::new_v4(),
            source: "test".to_string(),
            timestamp: Utc::now(),
            data: Some(serde_json::json!({"title": "Test Task"})),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&original_event).unwrap();

        // Deserialize back to Event
        let deserialized_event: Event = serde_json::from_str(&json).unwrap();

        assert_eq!(original_event.event_type, deserialized_event.event_type);
        assert_eq!(original_event.id, deserialized_event.id);
        assert_eq!(original_event.source, deserialized_event.source);
        assert_eq!(original_event.data, deserialized_event.data);
    }
}
