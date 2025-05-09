use sqlx::{SqlitePool, Row};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct WorldState {
    pub locations: Vec<Location>,
    pub factions: Vec<Faction>,
    pub npcs: Vec<Npc>,
    pub player: Player,
    pub world: World,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Location { pub id: i32, pub name: String, pub prosperity: i32, pub safety: i32 }
#[derive(Serialize, Deserialize, Clone)]
pub struct Faction { pub id: i32, pub name: String, pub power: i32, pub relation: String }
#[derive(Serialize, Deserialize, Clone)]
pub struct Npc { pub id: i32, pub name: String, pub role: String, pub status: String, pub location_id: i32 }
#[derive(Serialize, Deserialize, Clone)]
pub struct Player { pub id: i32, pub location_id: i32, pub reputation: i32 }
#[derive(Serialize, Deserialize, Clone)]
pub struct World { pub tension: i32, pub story_phase: String }

pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("CREATE TABLE IF NOT EXISTS locations (id INTEGER PRIMARY KEY, name TEXT, prosperity INTEGER, safety INTEGER)").execute(pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS factions (id INTEGER PRIMARY KEY, name TEXT, power INTEGER, relation TEXT)").execute(pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS npcs (id INTEGER PRIMARY KEY, name TEXT, role TEXT, status TEXT, location_id INTEGER)").execute(pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS player (id INTEGER PRIMARY KEY, location_id INTEGER, reputation INTEGER)").execute(pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS world (id INTEGER PRIMARY KEY, tension INTEGER, story_phase TEXT)").execute(pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS event_log (id INTEGER PRIMARY KEY AUTOINCREMENT, timestamp TEXT, description TEXT, caused_by TEXT)").execute(pool).await?;
    Ok(())
}

pub async fn populate_initial_data(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO locations (id, name, prosperity, safety) VALUES (1, 'Capital', 80, 90)").execute(pool).await?;
    sqlx::query("INSERT OR IGNORE INTO locations (id, name, prosperity, safety) VALUES (2, 'Willowbrook', 50, 60)").execute(pool).await?;
    sqlx::query("INSERT OR IGNORE INTO factions (id, name, power, relation) VALUES (1, 'Royal Guard', 70, 'Friendly')").execute(pool).await?;
    sqlx::query("INSERT OR IGNORE INTO factions (id, name, power, relation) VALUES (2, 'Bandits', 30, 'Hostile')").execute(pool).await?;
    sqlx::query("INSERT OR IGNORE INTO npcs (id, name, role, status, location_id) VALUES (1, 'King Alric', 'Ruler', 'Alive', 1)").execute(pool).await?;
    sqlx::query("INSERT OR IGNORE INTO player (id, location_id, reputation) VALUES (1, 1, 50)").execute(pool).await?;
    sqlx::query("INSERT OR IGNORE INTO world (id, tension, story_phase) VALUES (1, 20, 'Build-Up')").execute(pool).await?;
    Ok(())
}

pub async fn get_world_state(pool: &SqlitePool) -> Result<WorldState, sqlx::Error> {
    let locations = sqlx::query("SELECT * FROM locations").fetch_all(pool).await?
        .into_iter().map(|row| Location { id: row.get(0), name: row.get(1), prosperity: row.get(2), safety: row.get(3) }).collect();
    let factions = sqlx::query("SELECT * FROM factions").fetch_all(pool).await?
        .into_iter().map(|row| Faction { id: row.get(0), name: row.get(1), power: row.get(2), relation: row.get(3) }).collect();
    let npcs = sqlx::query("SELECT * FROM npcs").fetch_all(pool).await?
        .into_iter().map(|row| Npc { id: row.get(0), name: row.get(1), role: row.get(2), status: row.get(3), location_id: row.get(4) }).collect();
    let player = sqlx::query("SELECT * FROM player").fetch_one(pool).await?;
    let world = sqlx::query("SELECT * FROM world").fetch_one(pool).await?;
    Ok(WorldState {
        locations,
        factions,
        npcs,
        player: Player { id: player.get(0), location_id: player.get(1), reputation: player.get(2) },
        world: World { tension: world.get(1), story_phase: world.get(2) },
    })
}

pub async fn log_event(pool: &SqlitePool, description: &str, caused_by: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO event_log (timestamp, description, caused_by) VALUES (datetime('now'), ?, ?)")
        .bind(description).bind(caused_by).execute(pool).await?;
    Ok(())
}