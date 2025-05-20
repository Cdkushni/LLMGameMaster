use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct StoryEvent {
    pub id: i32,
    pub narrative: String,
    pub state_changes: serde_json::Value, // e.g., {"world.tension": 10, "locations.1.safety": -20}
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BranchChoice {
    pub id: i32,
    pub event_id: i32,
    pub description: String,
    pub state_changes: serde_json::Value, // e.g., {"factions.1.power": 5}
    pub is_default: bool,
}