mod models;

use axum::{
    routing::{get, post, put, delete},
    Router,
};

use std::sync::{Arc, Mutex};
use models::{Pokemon, CreatePokemon, UpdatePokemon};

// shared state: a list of Pokemon protected by a Mutex
type SharedState = Arc<Mutex<Vec<Pokemon>>>;

#[tokio::main]
async fn main() {
    // start with an empty team
    let state: SharedState = Arc::new(Mutex::new(Vec::new()));

    // build app with a router
    let app = Router::new()
        .route("/", get(root))
        .route("/pokemon", get(get_all_pokemon))
        .route("/pokemon", post(create_pokemon))
        .route("/pokemon/{id}", get(get_pokemon_by_id))
        .route("/pokemon/{id}", put(update_pokemon))
        .route("/pokemon/{id}", delete(delete_pokemon))
        .with_state(state);

    // run server on localhost:3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    println!("Server running on http://127.0.0.1:3000");
    println!("Try: curl http://localhost:3000/pokemon");

    axum::serve(listener, app).await.unwrap();
}

// health check main page function - not part of CRUD
async fn root() -> &'static str {
    "Pokemon Team API - Try GET /pokemon"
}

use axum::{
    extract::{State, Json, Path},
    http::StatusCode
};

// CREATE - Add a new Pokemon
// declare we have extractor for state + destructure
async fn create_pokemon(
    State(state): State<SharedState>,
    Json(payload): Json<CreatePokemon>,
    ) -> (StatusCode, Json<Pokemon>){
    let mut team = state.lock().unwrap();

    // create new id for indexing
    let new_id: u32 = if let Some(last_pokemon) = team.last(){
        last_pokemon.id + 1
    } else {
        1
    };

    let new_pokemon: Pokemon = Pokemon{
        id: new_id,
        name: payload.name,
        poke_type: payload.poke_type,
        level: payload.level,
    };

    // solves the problem of sending one copy to vec and the other back as payload    
    team.push(new_pokemon.clone());

    (StatusCode::CREATED, Json(new_pokemon))
}

// READ - get all pokemons
async fn get_all_pokemon(State(state): State<SharedState>) -> Json<Vec<Pokemon>>{
    let team = state.lock().unwrap();
    // can't move vector out of mutex so we clone
    Json(team.clone())
}

// READ - Get one Pokemon by ID
// return result since we might not find any id matching
async fn get_pokemon_by_id(State(state): State<SharedState>, Path(id): Path<u32>) -> Result<Json<Pokemon>, StatusCode> {
    let team = state.lock().unwrap();

    if let Some(pokemon) = team.iter().find(|p| p.id == id) {
        Ok(Json(pokemon.clone()))
    } else{
        Err(StatusCode::NOT_FOUND)
    }
}

async fn update_pokemon(
    State(state): State<SharedState>,
    Path(id): Path<u32>,
    Json(payload): Json<UpdatePokemon>)
    -> Result<Json<Pokemon>, StatusCode> {
    // update require mutable mutexguard (roleplaying as vec)
    let mut team = state.lock().unwrap();

    if let Some(pokemon) = team.iter_mut().find(|p| p.id == id){
        // any way we can reduce LOC here since we're just testing if not null
        if let Some(name) = payload.name{
            pokemon.name = name;
        }
        if let Some(poke_type) = payload.poke_type{
            pokemon.poke_type = poke_type;
        }
        if let Some(level) = payload.level{
            pokemon.level = level;
        }

        Ok(Json(pokemon.clone()))
    } else{
        Err(StatusCode::NOT_FOUND)
    }
}
async fn delete_pokemon(State(state): State<SharedState>, Path(id): Path<u32>) -> StatusCode {
    let mut team = state.lock().unwrap();
    let original_len = team.len();

    // retain = keep item that satisfies the following condition
    team.retain(|p| p.id != id);
    
    if team.len() < original_len {
        StatusCode::NO_CONTENT  // 204 - Successfully deleted
    } else {
        StatusCode::NOT_FOUND   // 404 - Pokemon wasn't there
    }
}