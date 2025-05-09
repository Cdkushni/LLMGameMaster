use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_cors::Cors; // Add this import
use sqlx::SqlitePool;
use serde_json::json;
use std::fs::File;
use std::path::Path;
use crate::db::{init_db, populate_initial_data, get_world_state};
use crate::game_master::GameMaster;

mod db;
mod game_master;

struct AppState {
    pool: SqlitePool,
    gm: GameMaster,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_path = "/home/colin/Repos/sci_fi_gm/world.db";
    println!("Attempting to connect to database at: {}", db_path);
    if !Path::new(db_path).exists() {
        println!("Database file does not exist, creating it...");
        File::create(db_path)?;
    } else {
        println!("Database file already exists");
    }

    let pool = match SqlitePool::connect(db_path).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to connect to database: {:?}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Database connection failed"));
        }
    };

    if let Err(e) = init_db(&pool).await {
        eprintln!("Failed to initialize database: {:?}", e);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Database initialization failed"));
    }
    if let Err(e) = populate_initial_data(&pool).await {
        eprintln!("Failed to populate initial data: {:?}", e);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Data population failed"));
    }

    let app_state = web::Data::new(AppState { pool, gm: GameMaster::new() });

    println!("Starting server at http://127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::default() // Default CORS policy
                .allowed_origin("http://127.0.0.1:8081") // Allow frontend origin
                .allowed_methods(vec!["GET", "POST"])
                .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE])
                .max_age(3600))
            .app_data(app_state.clone())
            .route("/state/update", web::post().to(update_state))
            .route("/world/state", web::get().to(world_state))
            .route("/events", web::get().to(get_events))
            .route("/reset", web::post().to(reset))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn update_state(data: web::Data<AppState>, state_change: web::Json<serde_json::Value>) -> impl Responder {
    let response = data.gm.update_world(&data.pool, state_change.into_inner()).await.unwrap();
    HttpResponse::Ok().json(response)
}

async fn world_state(data: web::Data<AppState>) -> impl Responder {
    let state = get_world_state(&data.pool).await.unwrap();
    HttpResponse::Ok().json(state)
}

async fn get_events(data: web::Data<AppState>) -> impl Responder {
    let response = data.gm.generate_events(&data.pool).await.unwrap();
    HttpResponse::Ok().json(response)
}

async fn reset(data: web::Data<AppState>) -> impl Responder {
    init_db(&data.pool).await.unwrap();
    populate_initial_data(&data.pool).await.unwrap();
    HttpResponse::Ok().json(json!({"message": "World reset"}))
}