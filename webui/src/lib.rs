use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
struct World {
    id: i32,
    tension: i32,
    story_phase: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Player {
    id: i32,
    location_id: i32,
    reputation: i32,
}

#[derive(Clone, Serialize, Deserialize)]
struct Location {
    id: i32,
    name: String,
    prosperity: i32,
    safety: i32,
}

#[derive(Clone, Serialize, Deserialize)]
struct Faction {
    id: i32,
    name: String,
    power: i32,
    relation: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Npc {
    id: i32,
    name: String,
    role: String,
    status: String,
    location_id: i32,
}

#[derive(Clone, Serialize, Deserialize)]
struct WorldState {
    world: World,
    player: Player,
    locations: Vec<Location>,
    factions: Vec<Faction>,
    npcs: Vec<Npc>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Event {
    id: i32,
    world_id: i32,
    description: String,
    created_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct GenerateEventRequest {
    context: String,
}

#[component]
pub fn App() -> impl IntoView {
    log::info!("Rendering Leptos App");
    
    let (world_state, set_world_state) = create_signal(None::<WorldState>);
    let (events, set_events) = create_signal(Vec::<Event>::new());
    let (error, set_error) = create_signal(None::<String>);
    let (context, set_context) = create_signal(String::new());

    let fetch_world = move |_| {
        log::info!("Fetching world state");
        spawn_local(async move {
            let response = reqwest::Client::new()
                .get("http://127.0.0.1:8080/world/state")
                .send()
                .await;
            
            log::info!("World response: {:?}", response);
            match response {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<WorldState>().await {
                        Ok(data) => set_world_state.set(Some(data)),
                        Err(e) => set_error.set(Some(format!("Failed to decode response: {}", e))),
                    }
                }
                Ok(resp) => set_error.set(Some(format!("Request failed with status {}", resp.status()))),
                Err(e) => set_error.set(Some(format!("Failed to send request: {}", e))),
            }
        });
    };

    let fetch_events = move |_| {
        log::info!("Fetching events");
        spawn_local(async move {
            let response = reqwest::Client::new()
                .get("http://127.0.0.1:8080/events")
                .send()
                .await;
            
            log::info!("Events response: {:?}", response);
            match response {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<Vec<Event>>().await {
                        Ok(data) => set_events.set(data),
                        Err(e) => set_error.set(Some(format!("Failed to decode events: {}", e))),
                    }
                }
                Ok(resp) => set_error.set(Some(format!("Events request failed with status {}", resp.status()))),
                Err(e) => set_error.set(Some(format!("Failed to fetch events: {}", e))),
            }
        });
    };

    let generate_event = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let context_value = context.get();
        if context_value.trim().is_empty() {
            set_error.set(Some("Context cannot be empty".to_string()));
            return;
        }
        log::info!("Generating event with context: {}", context_value);
        spawn_local(async move {
            let response = reqwest::Client::new()
                .post("http://127.0.0.1:8080/story/event")
                .json(&GenerateEventRequest { context: context_value.clone() })
                .send()
                .await;
            
            log::info!("Generate event response: {:?}", response);
            match response {
                Ok(resp) if resp.status().is_success() => {
                    set_context.set(String::new());
                    set_error.set(None);
                    // Refresh events
                    let events_response = reqwest::Client::new()
                        .get("http://127.0.0.1:8080/events")
                        .send()
                        .await;
                    match events_response {
                        Ok(resp) if resp.status().is_success() => {
                            match resp.json::<Vec<Event>>().await {
                                Ok(data) => set_events.set(data),
                                Err(e) => set_error.set(Some(format!("Failed to decode events: {}", e))),
                            }
                        }
                        Ok(resp) => set_error.set(Some(format!("Events request failed with status {}", resp.status()))),
                        Err(e) => set_error.set(Some(format!("Failed to fetch events: {}", e))),
                    }
                }
                Ok(resp) => set_error.set(Some(format!("Generate event failed with status {}", resp.status()))),
                Err(e) => set_error.set(Some(format!("Failed to send generate event request: {}", e))),
            }
        });
    };

    view! {
        <div>
            <h1>"Game Master UI"</h1>
            <button on:click=fetch_world>"Refresh World State"</button>
            <button on:click=fetch_events>"Refresh Events"</button>
            
            <form on:submit=generate_event>
                <input
                    type="text"
                    placeholder="Enter event context (e.g., battle between factions)"
                    value={context.get()}
                    on:input=move |ev| set_context.set(event_target_value(&ev))
                />
                <button type="submit">"Generate Event"</button>
            </form>
            
            {move || error.get().map(|err| view! {
                <p style="color: red;">{"Error: "}{err}</p>
            })}
            
            {move || world_state.get().map(|state| view! {
                <div>
                    <h2>"World State"</h2>
                    <p>{format!("World: {} (Tension: {})", state.world.story_phase, state.world.tension)}</p>
                    <p>{format!("Player: Reputation {}", state.player.reputation)}</p>
                    <h3>"Locations"</h3>
                    <ul>
                        {state.locations.into_iter().map(|loc| view! {
                            <li>{format!("{}: Prosperity {}, Safety {}", loc.name, loc.prosperity, loc.safety)}</li>
                        }).collect::<Vec<_>>()}
                    </ul>
                    <h3>"Factions"</h3>
                    <ul>
                        {state.factions.into_iter().map(|fac| view! {
                            <li>{format!("{}: Power {}, Relation {}", fac.name, fac.power, fac.relation)}</li>
                        }).collect::<Vec<_>>()}
                    </ul>
                    <h3>"NPCs"</h3>
                    <ul>
                        {state.npcs.into_iter().map(|npc| view! {
                            <li>{format!("{}: {} ({}) at Location {}", npc.name, npc.role, npc.status, npc.location_id)}</li>
                        }).collect::<Vec<_>>()}
                    </ul>
                </div>
            })}
            
            <h2>"Events"</h2>
            <ul>
                {move || events.get().into_iter().map(|event| view! {
                    <li>
                        <p>{format!("{} ({})", event.description, event.created_at)}</p>
                        <img src={format!("http://127.0.0.1:8080/event/image/{}", event.id)} alt={event.description.clone()} style="max-width: 256px;" />
                    </li>
                }).collect::<Vec<_>>()}
            </ul>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Starting Leptos app");
    leptos::mount_to_body(App);
}