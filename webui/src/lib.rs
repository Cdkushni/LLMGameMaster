use serde::{Deserialize, Serialize};
use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use reqwest::Client;
use web_sys::{HtmlSelectElement, HtmlInputElement};
use wasm_bindgen::prelude::{wasm_bindgen, JsCast, JsValue};
use std::rc::Rc;

#[derive(Serialize, Deserialize, Clone)]
struct WorldState {
    world: World,
    player: Player,
    locations: Vec<Location>,
    factions: Vec<Faction>,
    #[serde(default)]
    npcs: Vec<Npc>,
}

#[derive(Serialize, Deserialize, Clone)]
struct World {
    tension: i32,
    story_phase: String,
    #[serde(default)]
    id: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Player { id: i32, location_id: i32, reputation: i32 }

#[derive(Serialize, Deserialize, Clone)]
struct Location { id: i32, name: String, prosperity: i32, safety: i32 }

#[derive(Serialize, Deserialize, Clone)]
struct Faction {
    id: i32,
    name: String,
    power: i32,
    #[serde(default)]
    relation: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct Npc {
    id: i32,
    name: String,
    role: String,
    status: String,
    location_id: i32,
}

#[derive(Serialize, Deserialize)]
struct EventResponse { events: Vec<String>, narrative: String }

#[derive(Clone)]
struct AppState {
    world_state: Option<Rc<WorldState>>,
    events: Option<Rc<EventResponse>>,
    error: Option<String>,
}

#[function_component(App)]
fn app() -> Html {
    let state = use_state(|| AppState {
        world_state: None,
        events: None,
        error: None,
    });

    let refresh_world = {
        let state = state.clone();
        Callback::from(move |_| {
            let state = state.clone();
            spawn_local(async move {
                match Client::new().get("http://127.0.0.1:8080/world/state").send().await {
                    Ok(resp) => {
                        match resp.text().await {
                            Ok(text) => {
                                web_sys::console::log_1(&JsValue::from_str(&format!("Raw response: {}", text)));
                                match serde_json::from_str::<WorldState>(&text) {
                                    Ok(world) => state.set(AppState {
                                        world_state: Some(Rc::new(world)),
                                        events: (*state).events.clone(),
                                        error: None,
                                    }),
                                    Err(e) => state.set(AppState {
                                        world_state: (*state).world_state.clone(),
                                        events: (*state).events.clone(),
                                        error: Some(format!("Decoding error: {}", e)),
                                    }),
                                }
                            },
                            Err(e) => state.set(AppState {
                                world_state: (*state).world_state.clone(),
                                events: (*state).events.clone(),
                                error: Some(format!("Text error: {}", e)),
                            }),
                        }
                    },
                    Err(e) => state.set(AppState {
                        world_state: (*state).world_state.clone(),
                        events: (*state).events.clone(),
                        error: Some(e.to_string()),
                    }),
                }
            });
        })
    };

    let submit_action = {
        let state = state.clone();
        let refresh_world = refresh_world.clone();
        Callback::from(move |e: MouseEvent| {
            let state = state.clone();
            let refresh_world = refresh_world.clone();
            e.prevent_default();
            let action = web_sys::window().unwrap().document().unwrap()
                .get_element_by_id("action_select").unwrap()
                .dyn_into::<HtmlSelectElement>().unwrap().value();
            let target = web_sys::window().unwrap().document().unwrap()
                .get_element_by_id("target_input").unwrap()
                .dyn_into::<HtmlInputElement>().unwrap().value();
            spawn_local(async move {
                let body = serde_json::json!({"action": action, "target": target.parse::<i32>().unwrap_or(0)});
                match Client::new().post("http://127.0.0.1:8080/state/update")
                    .json(&body)
                    .send()
                    .await {
                    Ok(_) => {
                        state.set(AppState {
                            world_state: (*state).world_state.clone(),
                            events: (*state).events.clone(),
                            error: None,
                        });
                        refresh_world.emit(MouseEvent::new("click").unwrap());
                    },
                    Err(e) => state.set(AppState {
                        world_state: (*state).world_state.clone(),
                        events: (*state).events.clone(),
                        error: Some(e.to_string()),
                    }),
                }
            });
        })
    };

    let check_events = {
        let state = state.clone();
        Callback::from(move |_| {
            let state = state.clone();
            spawn_local(async move {
                match Client::new().get("http://127.0.0.1:8080/events").send().await {
                    Ok(resp) => match resp.json::<EventResponse>().await {
                        Ok(events) => state.set(AppState {
                            world_state: (*state).world_state.clone(),
                            events: Some(Rc::new(events)),
                            error: None,
                        }),
                        Err(e) => state.set(AppState {
                            world_state: (*state).world_state.clone(),
                            events: (*state).events.clone(),
                            error: Some(e.to_string()),
                        }),
                    },
                    Err(e) => state.set(AppState {
                        world_state: (*state).world_state.clone(),
                        events: (*state).events.clone(),
                        error: Some(e.to_string()),
                    }),
                }
            });
        })
    };

    // Auto-refresh on load (runs once)
    {
        let refresh_world = refresh_world.clone();
        use_effect_with((), move |_| {
            refresh_world.emit(MouseEvent::new("click").unwrap());
            || () // Cleanup closure
        });
    }

    html! {
        <div>
            <h1>{ "Game Master UI" }</h1>
            <button onclick={refresh_world}>{ "Refresh World State" }</button>
            <div>
                { match &state.world_state {
                    Some(world) => html! {
                        <pre>{ serde_json::to_string_pretty(&**world).unwrap_or("Error formatting".into()) }</pre>
                    },
                    None => html! { <p>{ "Loading..." }</p> },
                }}
            </div>
            <div>
                <select id="action_select">
                    <option value="move">{ "Move" }</option>
                    <option value="help">{ "Help" }</option>
                    <option value="fight">{ "Fight" }</option>
                </select>
                <input id="target_input" type="number" placeholder="Target ID" />
                <button onclick={submit_action}>{ "Submit" }</button>
            </div>
            <button onclick={check_events}>{ "Check Events" }</button>
            <div>
                { match &state.events {
                    Some(events) => html! {
                        <>
                            <p>{ "Events:" }</p>
                            <ul>{ for events.events.iter().map(|e| html! { <li>{ e }</li> }) }</ul>
                            <p>{ "Narrative: " }{ &events.narrative }</p>
                        </>
                    },
                    None => html! { <p>{ "No events yet" }</p> },
                }}
            </div>
            { if let Some(error) = &state.error {
                html! { <p style="color: red;">{ "Error: " }{ error }</p> }
            } else {
                html! {}
            }}
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}