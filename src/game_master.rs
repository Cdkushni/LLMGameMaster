use serde::{Serialize, Deserialize};
use sqlx::{SqlitePool, Row};
use rand::Rng;
use std::fs;
use crate::db::log_event;
use crate::db::get_world_state;

#[derive(Serialize, Deserialize)]
pub struct EventResponse { pub events: Vec<String>, pub narrative: String }

#[derive(Serialize, Deserialize)]
struct EventTemplate { phase: String, description: String, effect: Option<Effect> }
#[derive(Serialize, Deserialize)]
struct Effect { query: String, params: Vec<String> }

pub struct GameMaster {
    event_templates: Vec<EventTemplate>,
    story_cycles: serde_json::Value,
    tension_thresholds: Vec<(String, i32)>,
}

impl GameMaster {
    pub fn new() -> Self {
        // Read and parse events.json
        let event_content = match fs::read_to_string("data/events.json") {
            Ok(content) => content,
            Err(e) => panic!("Failed to read data/events.json: {:?}", e),
        };
        let event_templates: Vec<EventTemplate> = match serde_json::from_str(&event_content) {
            Ok(templates) => templates,
            Err(e) => panic!("Failed to parse data/events.json: {:?}", e),
        };

        // Read and parse story_cycles.json
        let story_content = match fs::read_to_string("data/story_cycles.json") {
            Ok(content) => content,
            Err(e) => panic!("Failed to read data/story_cycles.json: {:?}", e),
        };
        let story_cycles: serde_json::Value = match serde_json::from_str(&story_content) {
            Ok(cycles) => cycles,
            Err(e) => panic!("Failed to parse data/story_cycles.json: {:?}", e),
        };

        GameMaster {
            event_templates,
            story_cycles,
            tension_thresholds: vec![("Build-Up".to_string(), 40), ("Conflict".to_string(), 70), ("Climax".to_string(), 100)],
        }
    }

    pub async fn update_world(&self, pool: &SqlitePool, state_change: serde_json::Value) -> Result<EventResponse, sqlx::Error> {
        let action = state_change["action"].as_str().unwrap_or("");
        let target = state_change["target"].as_i64().unwrap_or(0) as i32;
        let value = state_change["value"].as_i64().unwrap_or(1) as i32;
        let caused_by = state_change["caused_by"].as_str().unwrap_or("Player");

        match action {
            "move" => {
                sqlx::query("UPDATE player SET location_id = ? WHERE id = 1").bind(target).execute(pool).await?;
                log_event(pool, &format!("Knight moved to location {}", target), caused_by).await?;
            }
            "help" => {
                sqlx::query("UPDATE locations SET prosperity = prosperity + ?, safety = safety + ? WHERE id = ?")
                    .bind(10 * value).bind(10 * value).bind(target).execute(pool).await?;
                sqlx::query("UPDATE player SET reputation = reputation + ? WHERE id = 1").bind(5 * value).execute(pool).await?;
                log_event(pool, &format!("Knight helped location {}, increasing prosperity and safety", target), caused_by).await?;
            }
            "fight" => {
                sqlx::query("UPDATE factions SET power = power - ? WHERE id = ?")
                    .bind(10 * value).bind(target).execute(pool).await?;
                sqlx::query("UPDATE player SET reputation = reputation + ? WHERE id = 1").bind(3 * value).execute(pool).await?;
                log_event(pool, &format!("Knight fought faction {}, reducing their power", target), caused_by).await?;
            }
            _ => {}
        }

        let mut tension = sqlx::query("SELECT tension FROM world WHERE id = 1").fetch_one(pool).await?.get::<i32, _>(0);
        tension = (tension + rand::thread_rng().gen_range(5..15)).min(100);
        let current_phase = sqlx::query("SELECT story_phase FROM world WHERE id = 1").fetch_one(pool).await?.get::<String, _>(0);
        let new_phase = self.get_next_phase(tension, &current_phase);
        sqlx::query("UPDATE world SET tension = ?, story_phase = ? WHERE id = 1").bind(tension).bind(&new_phase).execute(pool).await?;

        Ok(self.generate_events(pool).await?)
    }

    fn get_next_phase(&self, tension: i32, current_phase: &str) -> String {
        for (phase, threshold) in &self.tension_thresholds {
            if tension < *threshold && phase != current_phase {
                return phase.clone();
            }
        }
        if tension >= 100 { "Climax".to_string() } else { current_phase.to_string() }
    }

    pub async fn generate_events(&self, pool: &SqlitePool) -> Result<EventResponse, sqlx::Error> {
        let state = get_world_state(pool).await?;
        let _tension = state.world.tension;
        let phase = state.world.story_phase.clone();
        let player_location = state.player.location_id;

        let possible_events: Vec<_> = self.event_templates.iter().filter(|e| e.phase == phase).collect();
        if possible_events.is_empty() {
            return Ok(EventResponse { events: vec![], narrative: "The kingdom is quiet for now...".to_string() });
        }

        let event = possible_events[rand::thread_rng().gen_range(0..possible_events.len())];
        let location_name = state.locations.iter().find(|l| l.id == player_location).unwrap().name.clone();
        let faction_name = state.factions[rand::thread_rng().gen_range(0..possible_events.len())].name.clone();
        let event_desc = event.description.replace("{location}", &location_name).replace("{faction}", &faction_name);
        log_event(pool, &event_desc, "System").await?;

        if let Some(effect) = &event.effect {
            let mut query = effect.query.clone();
            for param in &effect.params {
                query = query.replace(param, &state.locations.iter().find(|l| l.id == player_location).unwrap().name);
            }
            sqlx::query(&query).execute(pool).await?;
        }

        let narrative = self.story_cycles.get(&phase).unwrap_or(&serde_json::Value::String("The story unfolds...".to_string())).as_str().unwrap().to_string();
        Ok(EventResponse { events: vec![event_desc], narrative })
    }
}