use actix_web::{web, App, HttpResponse, HttpServer, Responder, Error};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, FromRow, Row};
use log::info;
use reqwest::Client;
use std::env;
use actix_web::body::to_bytes;

#[derive(Clone)]
struct AppState {
    pool: Pool<Sqlite>,
    client: Client,
}

#[derive(Serialize, Deserialize, FromRow)]
struct World {
    id: i32,
    tension: i32,
    story_phase: String,
}

#[derive(Serialize, Deserialize, FromRow)]
struct Player {
    id: i32,
    location_id: i32,
    reputation: i32,
}

#[derive(Serialize, Deserialize, FromRow)]
struct Location {
    id: i32,
    name: String,
    prosperity: i32,
    safety: i32,
}

#[derive(Serialize, Deserialize, FromRow)]
struct Faction {
    id: i32,
    name: String,
    power: i32,
    relation: String,
}

#[derive(Serialize, Deserialize, FromRow)]
struct Npc {
    id: i32,
    name: String,
    role: String,
    status: String,
    location_id: i32,
}

#[derive(Serialize, Deserialize)]
struct WorldState {
    world: World,
    player: Player,
    locations: Vec<Location>,
    factions: Vec<Faction>,
    npcs: Vec<Npc>,
}

#[derive(Serialize, Deserialize, FromRow)]
struct Event {
    id: i32,
    world_id: i32,
    description: String,
    created_at: String,
}

#[derive(Serialize, Deserialize)]
struct UpdateStateRequest {
    player_reputation: Option<i32>,
    faction_power: Option<Vec<(i32, i32)>>, // (faction_id, power)
}

#[derive(Serialize, Deserialize)]
struct GenerateEventRequest {
    context: String, // e.g., "battle between factions"
}

#[derive(Serialize, Deserialize)]
struct BranchChoice {
    id: i32,
    description: String,
}

#[actix_web::options("/world/state")]
async fn world_state_options() -> impl Responder {
    info!("Handling /world/state, method: OPTIONS");
    HttpResponse::Ok()
        .insert_header((actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "http://127.0.0.1:8081"))
        .insert_header((actix_web::http::header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD, OPTIONS"))
        .insert_header((actix_web::http::header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, Authorization, Accept"))
        .insert_header((actix_web::http::header::ACCESS_CONTROL_MAX_AGE, "3600"))
        .finish()
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

async fn get_world_state(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let pool = &data.pool;

    let world = sqlx::query_as::<_, World>("SELECT * FROM world WHERE id = 1")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch world: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch world state")
        })?;

    let player = sqlx::query_as::<_, Player>("SELECT * FROM player WHERE id = 1")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch player: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch player state")
        })?;

    let locations = sqlx::query_as::<_, Location>("SELECT * FROM locations")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch locations: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch locations")
        })?;

    let factions = sqlx::query_as::<_, Faction>("SELECT * FROM factions")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch factions: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch factions")
        })?;

    let npcs = sqlx::query_as::<_, Npc>("SELECT * FROM npcs")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch npcs: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch npcs")
        })?;

    let world_state = WorldState {
        world,
        player,
        locations,
        factions,
        npcs,
    };

    Ok(HttpResponse::Ok().json(world_state))
}

async fn update_state(data: web::Data<AppState>, req: web::Json<UpdateStateRequest>) -> Result<HttpResponse, Error> {
    let pool = &data.pool;

    if let Some(reputation) = req.player_reputation {
        sqlx::query("UPDATE player SET reputation = ? WHERE id = 1")
            .bind(reputation)
            .execute(pool)
            .await
            .map_err(|e| {
                log::error!("Failed to update player reputation: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to update player")
            })?;
    }

    if let Some(faction_powers) = &req.faction_power {
        for (faction_id, power) in faction_powers {
            sqlx::query("UPDATE factions SET power = ? WHERE id = ?")
                .bind(power)
                .bind(faction_id)
                .execute(pool)
                .await
                .map_err(|e| {
                    log::error!("Failed to update faction {} power: {}", faction_id, e);
                    actix_web::error::ErrorInternalServerError("Failed to update faction")
                })?;
        }
    }

    Ok(HttpResponse::Ok().body("State updated"))
}

async fn get_events(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let pool = &data.pool;

    let events = sqlx::query_as::<_, Event>("SELECT * FROM events ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch events: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch events")
        })?;

    Ok(HttpResponse::Ok().json(events))
}

async fn generate_story_event(data: web::Data<AppState>, req: web::Json<GenerateEventRequest>) -> Result<HttpResponse, Error> {
    let pool = &data.pool;
    let client = &data.client;

    // Call Grok API for narrative
    let grok_api_key = env::var("GROK_API_KEY").map_err(|_| {
        log::error!("GROK_API_KEY not set");
        actix_web::error::ErrorInternalServerError("Missing GROK_API_KEY")
    })?;

    let mut description = "A mysterious event occurred.".to_string();
    
    let grok_response = client
        .post("https://api.x.ai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", grok_api_key))
        .json(&serde_json::json!({
            "model": "grok-3",
            "messages": [{
                "role": "user",
                "content": format!("Generate a sci-fi story event based on: {}. Keep it concise, under 100 words.", req.context)
            }]
        }))
        .send()
        .await;

    match grok_response {
        Ok(resp) => {
            let grok_status = resp.status();
            info!("Grok API response status: {}", grok_status);
            if grok_status.is_success() {
                let grok_json: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| {
                        log::error!("Failed to parse Grok response: {}", e);
                        actix_web::error::ErrorInternalServerError("Failed to parse Grok response")
                    })?;
                description = grok_json["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("A mysterious event occurred.")
                    .to_string();
            } else {
                let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
                log::warn!("Grok API failed (status {}), using fallback description: {}", grok_status, error_text);
            }
        }
        Err(e) => {
            log::warn!("Grok API request failed, using fallback description: {}", e);
        }
    }

    // Call Stability AI API for pixel art
    let stability_api_key = env::var("STABILITY_API_KEY").map_err(|_| {
        log::error!("STABILITY_API_KEY not set");
        actix_web::error::ErrorInternalServerError("Missing STABILITY_API_KEY")
    })?;

    let image_response = client
        .post("https://api.stability.ai/v1/generation/stable-diffusion-xl-1024-v1-0/text-to-image")
        .header("Authorization", format!("Bearer {}", stability_api_key))
        .header("Accept", "image/png")
        .json(&serde_json::json!({
            "text_prompts": [{
                "text": format!("16-bit pixel art of {}, retro style, vibrant colors, sharp edges", req.context),
                "weight": 1
            }],
            "cfg_scale": 7,
            "height": 1024,
            "width": 1024,
            "samples": 1,
            "steps": 30
        }))
        .send()
        .await;

    let image_data = match image_response {
        Ok(resp) => {
            let status = resp.status();
            info!("Stability AI response status: {}", status);
            if !status.is_success() {
                let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
                log::error!("Stability AI failed: {}", error_text);
                return Err(actix_web::error::ErrorInternalServerError(format!("Stability AI failed: {}", error_text)));
            }
            let bytes = resp.bytes().await.map_err(|e| {
                log::error!("Failed to read image data: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to read image data")
            })?;
            info!("Received image data: {} bytes", bytes.len());
            if bytes.is_empty() || bytes.len() < 1000 {
                log::error!("Invalid image data: {} bytes", bytes.len());
                return Err(actix_web::error::ErrorInternalServerError("Invalid image data"));
            }
            bytes.to_vec()
        }
        Err(e) => {
            log::error!("Stability AI request failed: {}", e);
            return Err(actix_web::error::ErrorInternalServerError(format!("Stability AI request failed: {}", e)));
        }
    };

    // Store event and image in database
    let event = sqlx::query_as::<_, Event>(
        "INSERT INTO events (world_id, description) VALUES (1, ?) RETURNING *"
    )
    .bind(&description)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to insert event: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to store event")
    })?;

    sqlx::query("INSERT INTO event_images (event_id, image_data) VALUES (?, ?)")
        .bind(event.id)
        .bind(&image_data)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to insert event image for event {}: {}", event.id, e);
            actix_web::error::ErrorInternalServerError("Failed to store event image")
        })?;

    info!("Stored event {} with image of {} bytes", event.id, image_data.len());

    Ok(HttpResponse::Ok().json(event))
}

async fn get_branch_choices(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let client = &data.client;

    // Fetch and deserialize world state
    let world_state_response = get_world_state(data.clone()).await?;
    let world_state: WorldState = serde_json::from_slice(
        &to_bytes(world_state_response.into_body())
            .await
            .map_err(|e| {
                log::error!("Failed to read response body: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to read response body")
            })?
    ).map_err(|e| {
        log::error!("Failed to deserialize world state: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to deserialize world state")
    })?;

    let world_state_json = serde_json::to_string(&world_state).map_err(|e| {
        log::error!("Failed to serialize world state: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to serialize world state")
    })?;

    // Call Grok API for choices
    let grok_api_key = env::var("GROK_API_KEY").map_err(|_| {
        log::error!("GROK_API_KEY not set");
        actix_web::error::ErrorInternalServerError("Missing GROK_API_KEY")
    })?;

    let grok_response = client
        .post("https://api.x.ai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", grok_api_key))
        .json(&serde_json::json!({
            "model": "grok-3",
            "messages": [{
                "role": "user",
                "content": format!("Based on this world state: {}. Generate 3 concise player decision options (each under 20 words) for the next story event.", world_state_json)
            }]
        }))
        .send()
        .await
        .map_err(|e| {
            log::error!("Grok API request failed: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to call Grok API")
        })?;

    let grok_status = grok_response.status();
    info!("Grok API response status: {}", grok_status);
    if !grok_status.is_success() {
        let error_text = grok_response.text().await.unwrap_or("Unknown error".to_string());
        log::error!("Grok API failed: {}", error_text);
        return Err(actix_web::error::ErrorInternalServerError(format!("Grok API failed: {}", error_text)));
    }

    let grok_json: serde_json::Value = grok_response
        .json()
        .await
        .map_err(|e| {
            log::error!("Failed to parse Grok response: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to parse Grok response")
        })?;

    let choices_text = grok_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("1. Explore ruins.\n2. Negotiate peace.\n3. Attack bandits.")
        .split('\n')
        .enumerate()
        .map(|(i, desc)| BranchChoice {
            id: (i + 1) as i32,
            description: desc.trim().to_string(),
        })
        .collect::<Vec<BranchChoice>>();

    Ok(HttpResponse::Ok().json(choices_text))
}

async fn get_event_image(data: web::Data<AppState>, path: web::Path<i32>) -> Result<HttpResponse, Error> {
    let pool = &data.pool;
    let event_id = path.into_inner();

    let image_data = sqlx::query("SELECT image_data FROM event_images WHERE event_id = ?")
        .bind(event_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch image for event {}: {}", event_id, e);
            actix_web::error::ErrorNotFound("Image not found")
        })?
        .get::<Vec<u8>, _>("image_data");

    info!("Serving image for event {}: {} bytes", event_id, image_data.len());

    if image_data.is_empty() || image_data.len() < 1000 {
        log::error!("Invalid image data for event {}: {} bytes", event_id, image_data.len());
        return Err(actix_web::error::ErrorInternalServerError("Invalid image data"));
    }

    Ok(HttpResponse::Ok()
        .content_type("image/png")
        .body(image_data))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let pool = sqlx::sqlite::SqlitePool::connect("sqlite://world.db").await.unwrap();
    let client = Client::new();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://127.0.0.1:8081")
            .allowed_methods(vec!["GET", "HEAD", "OPTIONS", "POST"])
            .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE, actix_web::http::header::AUTHORIZATION, actix_web::http::header::ACCEPT])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(AppState {
                pool: pool.clone(),
                client: client.clone(),
            }))
            .service(world_state_options)
            .service(web::resource("/health").route(web::get().to(health_check)).route(web::head().to(health_check)))
            .service(web::resource("/world/state").route(web::get().to(get_world_state)).route(web::head().to(get_world_state)))
            .service(web::resource("/state").route(web::post().to(update_state)))
            .service(web::resource("/events").route(web::get().to(get_events)))
            .service(web::resource("/story/event").route(web::post().to(generate_story_event)))
            .service(web::resource("/branch/choices").route(web::get().to(get_branch_choices)))
            .service(web::resource("/event/image/{id}").route(web::get().to(get_event_image)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}