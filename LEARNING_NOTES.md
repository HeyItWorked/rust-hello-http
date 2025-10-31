# Rust Web Development Learning Notes 📚

A collection of key concepts, patterns, and mental models for building CRUD APIs with Rust.

---

## Table of Contents
1. [Data Models & DTOs](#data-models--dtos)
2. [Static Lifetimes](#static-lifetimes)
3. [Shared State Pattern](#shared-state-pattern)
4. [REST API Routing](#rest-api-routing)
5. [Path Parameters](#path-parameters)
6. [Axum Extractors](#axum-extractors)
7. [Extractor Pattern Deep Dive](#extractor-pattern-deep-dive)
8. [MutexGuard Explained](#mutexguard-explained)
9. [Clone vs Move in CRUD](#clone-vs-move-in-crud)
10. [Result Type & Error Handling](#result-type--error-handling)
11. [Iterators & Closures](#iterators--closures)
12. [Vec<T> Quick Reference](#vect-quick-reference)
13. [Mutable vs Immutable Methods](#mutable-vs-immutable-methods)
14. [CRUD Function Design](#crud-function-design)

---

## Data Models & DTOs

### The Three-Struct Pattern

When building CRUD APIs, use separate structs for different purposes:

```rust
// 1. DOMAIN MODEL - The actual data entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pokemon {
    pub id: u32,           // Server-generated
    pub name: String,
    pub poke_type: String,
    pub level: u32,
}

// 2. CREATE DTO - For accepting creation requests
#[derive(Debug, Deserialize)]
pub struct CreatePokemon {
    // No `id` - server generates it!
    pub name: String,
    pub poke_type: String,
    pub level: u32,
}

// 3. UPDATE DTO - For accepting update requests
#[derive(Debug, Deserialize)]
pub struct UpdatePokemon {
    // All fields Optional for partial updates
    pub name: Option<String>,
    pub poke_type: Option<String>,
    pub level: Option<u32>,
}
```

### Why This Pattern?

| Struct | Purpose | Key Feature |
|--------|---------|-------------|
| **Domain Model** | Persistence/storage | Has ALL fields including `id` |
| **CreateDTO** | POST requests | Excludes `id` (server generates) |
| **UpdateDTO** | PUT/PATCH requests | All fields `Option<T>` (partial updates) |

**Benefits:**
- Type safety - impossible to accept client-specified IDs on creation
- Flexibility - clients can update just one field
- Separation of concerns - what clients send vs. what you store

---

## Static Lifetimes

### What is `&'static`?

The `'static` lifetime means data lives for the **entire duration of the program**.

```rust
async fn root() -> &'static str {
    "Hello, World!"  // ← Compiled into the binary
}
```

### Mental Models

#### Explain Like You're 5 🎈
- **Regular reference (`&str`)**: A sticker on your lunchbox. Throw away the lunchbox → sticker is gone.
- **Static reference (`&'static str`)**: A sticker printed in a library book. It's ALWAYS there, forever!

#### Explain Like a Programmer 💻

| Type | Lifetime | Where It Lives |
|------|----------|----------------|
| `&'static str` | Forever | Compiled into binary |
| `String` | Until dropped | Heap (runtime allocated) |
| `&str` | Depends | Points to String or `&'static str` |

### When You Need `&'static`

```rust
// ❌ ERROR - how long does this reference live?
async fn root() -> &str {
    "Hello"
}

// ✅ OK - lives forever!
async fn root() -> &'static str {
    "Hello"
}

// ✅ Also OK - returns owned data (no lifetime needed)
async fn root() -> String {
    "Hello".to_string()
}
```

**Rule of thumb:** If you're returning a string literal from a function, use `&'static str` or convert to `String`.

---

## Shared State Pattern

### The Arc + Mutex Pattern

For thread-safe shared state in web servers:

```rust
use std::sync::{Arc, Mutex};

// Type alias for cleaner code
type SharedState = Arc<Mutex<Vec<Pokemon>>>;

#[tokio::main]
async fn main() {
    // Create shared state
    let state: SharedState = Arc::new(Mutex::new(Vec::new()));
    
    // Inject into router
    let app = Router::new()
        .route("/pokemon", get(get_all_pokemon))
        .with_state(state);  // ← All handlers get access
}
```

### What Each Part Does

| Component | Purpose |
|-----------|---------|
| `Vec<Pokemon>` | The actual data structure (growable array) |
| `Mutex<T>` | Ensures thread-safe access (only one handler modifies at a time) |
| `Arc<T>` | Allows multiple handlers to share ownership of the same data |

### Why This Pattern?

**The Problem:** Web servers handle multiple requests concurrently. Without synchronization, you'd get data races.

**The Solution:** 
- `Mutex` prevents simultaneous writes
- `Arc` allows safe sharing across threads
- Rust's type system enforces this at compile time!

### State Injection Across Frameworks

This pattern is universal (with different syntax):

**Rust (Axum):**
```rust
.with_state(state)
// Handler: State(state): State<SharedState>
```

**Node.js (Express):**
```javascript
app.use((req, res, next) => {
    req.db = database;
    next();
});
```

**Python (FastAPI):**
```python
@app.get("/pokemon")
async def get_pokemon(db: Session = Depends(get_db)):
    pass
```

**What Makes Rust Different?**
Rust makes thread-safety **explicit** (`Arc<Mutex<T>>`). Other languages hide this behind garbage collection and runtime locks.

---

## REST API Routing

### The 5-Operation CRUD Pattern

For any resource (Pokemon, Users, Products), you have **5 core operations**:

| Operation | HTTP Method | Route Pattern | What It Does |
|-----------|-------------|---------------|--------------|
| **C**reate | `POST` | `/pokemon` | Add new item to collection |
| **R**ead All | `GET` | `/pokemon` | Get entire collection |
| **R**ead One | `GET` | `/pokemon/:id` | Get specific item |
| **U**pdate | `PUT/PATCH` | `/pokemon/:id` | Modify specific item |
| **D**elete | `DELETE` | `/pokemon/:id` | Remove specific item |

### The Mental Model 🧠

**Think of it like a filing cabinet:**

```
/pokemon           ← The whole drawer (collection)
/pokemon/:id       ← A specific file (individual item)
```

**URL = "What" | HTTP Method = "How"**

```
GET    /pokemon     →  "Show me the drawer"
POST   /pokemon     →  "Add a new file to the drawer"
GET    /pokemon/1   →  "Show me file #1"
PUT    /pokemon/1   →  "Replace/update file #1"
DELETE /pokemon/1   →  "Throw away file #1"
```

### Recognition Rules

1. **Same URL, different methods = different operations**
   ```
   POST   /pokemon    ← Create
   GET    /pokemon    ← Read all
   ```

2. **Has `:id`? = Operating on ONE specific item**
   ```
   GET    /pokemon/:id    ← Read ONE
   PUT    /pokemon/:id    ← Update ONE
   DELETE /pokemon/:id    ← Delete ONE
   ```

3. **No `:id`? = Operating on the WHOLE collection**
   ```
   GET  /pokemon    ← Read ALL
   POST /pokemon    ← Create (adds to collection)
   ```

### Example: Complete CRUD Routes

```rust
let app = Router::new()
    .route("/pokemon", post(create_pokemon))       // CREATE
    .route("/pokemon", get(get_all_pokemon))       // READ all
    .route("/pokemon/:id", get(get_pokemon_by_id)) // READ one
    .route("/pokemon/:id", put(update_pokemon))    // UPDATE
    .route("/pokemon/:id", delete(delete_pokemon)) // DELETE
    .with_state(state);
```

---

## Path Parameters

### What is `:id`?

The `:` creates a **variable placeholder** in your route:

```rust
.route("/pokemon/:id", get(get_pokemon_by_id))
```

`:id` means: **"Capture whatever value the user puts here and give it to me as a variable called `id`"**

### Examples

| User Requests | Route Match | `id` Value |
|---------------|-------------|------------|
| `/pokemon/1` | ✅ Yes | `1` |
| `/pokemon/25` | ✅ Yes | `25` |
| `/pokemon/9999` | ✅ Yes | `9999` |
| `/pokemon/` | ❌ No | - |
| `/pokemon/1/stats` | ❌ No | - |

### Multiple Parameters

```rust
.route("/users/:user_id/posts/:post_id", get(handler))

async fn handler(Path((user_id, post_id)): Path<(u32, u32)>) {
    // GET /users/5/posts/42  →  user_id=5, post_id=42
}
```

### Do Names Need to Match?

**No, but they should for readability!**

```rust
.route("/users/:user_id/posts/:post_id", get(handler))

// ✅ Readable (recommended)
async fn handler(Path((user_id, post_id)): Path<(u32, u32)>) { }

// ✅ Works but confusing
async fn handler(Path((foo, bar)): Path<(u32, u32)>) { }
```

**What matters:** Order and type, not names!

Axum extracts by **position**:
```
URL: /users/5/posts/42
Route: /users/:user_id/posts/:post_id
                 ↓              ↓
              Position 0     Position 1
Path((a, b)):    5             42
```

---

## Axum Extractors

### What are Extractors?

Extractors are Axum's way of pulling data from HTTP requests.

**Think of them as function arguments that auto-populate from the request.**

### Common Extractors

| Extractor | What It Extracts | Example |
|-----------|------------------|---------|
| `Path(id)` | URL path parameters (`:id`) | `/pokemon/:id` |
| `Json(data)` | JSON body from POST/PUT | Request body |
| `State(state)` | Shared application state | Your `Arc<Mutex<Vec>>` |
| `Query(params)` | Query strings | `/search?name=pikachu` |

### Combining Multiple Extractors

You can use multiple extractors in one handler:

```rust
async fn update_pokemon(
    State(state): State<SharedState>,       // ← Shared state
    Path(id): Path<u32>,                     // ← URL parameter
    Json(payload): Json<UpdatePokemon>,      // ← JSON body
) -> Result<Json<Pokemon>, StatusCode> {
    // You have: state, id, and payload!
}
```

**Order doesn't matter** - Axum figures out what each extractor needs!

### Error Handling

If extraction fails, Axum automatically returns an error:

```rust
Path(id): Path<u32>
```

| Request | Result |
|---------|--------|
| `GET /pokemon/42` | ✅ `id = 42` |
| `GET /pokemon/abc` | ❌ 400 Bad Request (can't parse as u32) |
| `GET /pokemon/` | ❌ 404 Not Found (no :id provided) |

---

## Extractor Pattern Deep Dive

### Why `State(state): State<SharedState>` Instead of `state: SharedState`?

This confuses many beginners! Let's break it down:

```rust
// ❌ This DOESN'T work in Axum handlers
async fn create_pokemon(state: SharedState) { }

// ✅ This DOES work - using the State extractor
async fn create_pokemon(State(state): State<SharedState>) { }
```

### The Pattern Matching Destructuring

```rust
State(state): State<SharedState>
  ↑     ↑         ↑
  |     |         └─ Type annotation (what Axum expects)
  |     └─────────── The actual variable you'll use
  └───────────────── Pattern destructuring
```

**Breaking it down:**

1. `State<SharedState>` - The **extractor type** Axum recognizes
2. `State(state)` - **Unwrap** the inner value and bind it to variable `state`
3. Now you use just `state` in your function body

### How Extractors Work

Axum's `State` is actually a **wrapper struct**:

```rust
// Simplified version of what Axum does internally
pub struct State<T>(T);  // Tuple struct wrapping your actual state
```

When you write:
```rust
State(state): State<SharedState>
```

You're saying:
- "Give me a `State<SharedState>` extractor"
- "Destructure it to get the inner `SharedState`"
- "Bind that inner value to the variable `state`"

### Visual Breakdown

```rust
async fn create_pokemon(
    State(state): State<SharedState>,
    //    └─┬──┘        └──────┬──────┘
    //      │                  │
    //      │                  └─ The type Axum expects
    //      └──────────────────── The variable you use
) {
    // Inside function, you just use `state` (not `State(state)`)
    let mut team = state.lock().unwrap();
    team.push(new_pokemon);
}
```

### Other Extractors Follow the Same Pattern

#### Path Extractor:
```rust
Path(id): Path<u32>
//   └─┬─┘     └──┬──┘
//     │          └─ Type Axum expects
//     └──────────── Variable you use
```

#### Json Extractor:
```rust
Json(payload): Json<CreatePokemon>
//   └───┬───┘      └────────┬───────┘
//       │                   └─ Type Axum expects
//       └─────────────────────── Variable you use
```

### Why Not Just `state: SharedState`?

Because Axum needs to know **"this is an extractor"** vs. **"this is a regular parameter"**.

```rust
// ❌ Axum doesn't know how to get this
async fn handler(state: SharedState) { }

// ✅ Axum sees State<T> extractor and knows:
//    "Ah, I need to inject the app state here!"
async fn handler(State(state): State<SharedState>) { }
```

### It's Like Other Languages, But Explicit

#### Express (JavaScript):
```javascript
// Middleware injects req.db
app.get('/pokemon', (req, res) => {
    const db = req.db;  // Magic injection
});
```

#### FastAPI (Python):
```python
# Depends() is the extractor
@app.get("/pokemon")
async def get_pokemon(db: Session = Depends(get_db)):
    pass
```

#### Axum (Rust):
```rust
// State() is the extractor - but explicit!
async fn get_pokemon(State(state): State<SharedState>) {
    // state is now available
}
```

**Rust makes it explicit** - you can see exactly what's being extracted and how.

### Pro Tip: You Can Rename the Variable

```rust
// Both work identically:
State(state): State<SharedState>
State(my_state): State<SharedState>
State(pokemon_store): State<SharedState>

// You're just choosing the variable name after destructuring
```

### Summary

| Syntax | Purpose |
|--------|---------||
| `State<SharedState>` | Type - tells Axum "this is a state extractor" |
| `State(state)` | Destructuring - unwraps the inner value |
| `state` | The actual variable you use in your function |

**It's not `state: SharedState` because Axum wouldn't know it needs to inject the state!**

---

## MutexGuard Explained

### What Happens When You Lock?

```rust
let mut team = state.lock().unwrap();
//      ↑            ↑        ↑
//      │            │        └─ "Please don't crash, just panic if locked"
//      │            └────────── "Give me exclusive access"
//      └─────────────────────── team is a MutexGuard, NOT a Vec!
```

### Explain Like You're 5 🎈

Imagine you have a **toy box** (your `Vec<Pokemon>`).

Your parents put a **lock** on it (the `Mutex`) so you and your sibling don't fight over toys.

When you want to play:
1. You ask for the **key** → `state.lock()`
2. You get a **special bracelet** that holds the key → `MutexGuard`
3. While you wear the bracelet, **only YOU can touch the toys** → exclusive access
4. When you're done, you **drop the bracelet** → the lock automatically closes!

**The bracelet (`MutexGuard`) proves you have permission to touch the toys!**

### Explain Like a Programmer 💻

```rust
let state: Arc<Mutex<Vec<Pokemon>>> = ...;
//                ↑         ↑
//                │         └─ Your actual data
//                └─────────── The lock protecting it

let team = state.lock().unwrap();
//         └────┬────┘  └──┬──┘
//              │          └─ Handle errors (convert Result to value)
//              └─ Acquire the lock (blocks if someone else has it)

// team is now: MutexGuard<'_, Vec<Pokemon>>
//                        ↑         ↑
//                        │         └─ The data inside
//                        └─ Lifetime (how long you can hold it)
```

### What is `MutexGuard<'_, Vec<Pokemon>>`?

#### The Layers:

```
Arc<Mutex<Vec<Pokemon>>>
    └─ Mutex<Vec<Pokemon>>      ← Lock around data
         └─ Vec<Pokemon>         ← Your actual data

When you call .lock():
MutexGuard<'_, Vec<Pokemon>>    ← Temporary "key holder"
           └─ Vec<Pokemon>       ← Can access this while holding guard
```

#### Breaking Down the Type:

```rust
MutexGuard<'_, Vec<Pokemon>>
│          │   │
│          │   └─ Type of data being guarded
│          └───── Lifetime (how long this guard is valid)
└──────────────── The guard type (smart pointer)
```

### Why Not Just `Vec<Pokemon>`?

Because Rust needs to **track ownership** and **automatically unlock**:

```rust
{
    let mut team = state.lock().unwrap();
    // team is MutexGuard<'_, Vec<Pokemon>>
    
    team.push(new_pokemon);  // Modifying the Vec
    
} // ← Guard drops here - lock AUTOMATICALLY released!

// Lock is now free for other requests to use
```

**The `MutexGuard` is like a smart bracelet:**
- While you wear it, you have exclusive access
- When you drop it (end of scope), the lock releases automatically
- No manual unlock needed - Rust handles it!

### The `'_` Lifetime

```rust
MutexGuard<'_, Vec<Pokemon>>
           ↑
           └─ "Some lifetime"
```

This means: **"This guard is only valid while the Mutex exists."**

Think of it as: "You can only hold the key while the lock still exists!"

Rust infers this automatically - you don't need to write it yourself.

### Using the Guard

Even though `team` is a `MutexGuard`, you can use it **like a `Vec`**:

```rust
let mut team = state.lock().unwrap();

// All these work!
team.push(new_pokemon);              // Add item
team.len();                          // Get length
team.iter();                         // Iterate
team.retain(|p| p.id != id);         // Filter

// MutexGuard implements Deref, so it "acts like" a Vec<Pokemon>
```

This is called **deref coercion** - the guard pretends to be the inner type!

### Visual Representation

```
Before .lock():
┌─────────────────┐
│ Arc<Mutex<...>> │ ← Shared, locked
│   🔒 LOCKED      │
│   [Pikachu,     │
│    Charizard]   │
└─────────────────┘

After .lock().unwrap():
┌─────────────────┐
│ MutexGuard      │ ← You hold the key!
│   🔓 UNLOCKED   │
│   (only you)    │
│                 │
│   Gives access  │
│   to Vec ↓      │
│                 │
│   [Pikachu,     │ ← Can modify now
│    Charizard]   │
└─────────────────┘

When guard drops:
┌─────────────────┐
│ Arc<Mutex<...>> │
│   🔒 RE-LOCKED  │ ← Automatically!
│   [Pikachu,     │
│    Charizard,   │
│    Raichu]      │ ← Changes saved
└─────────────────┘
```

### Common Pattern - Best Practice

```rust
// ✅ Good - Short-lived lock
{
    let mut team = state.lock().unwrap();
    team.push(new_pokemon);
    let count = team.len();
} // Lock released immediately

// ❌ Bad - holds lock too long
let mut team = state.lock().unwrap();
team.push(new_pokemon);
// ... lots of other code ...
// Lock still held! Other requests blocked!
```

**Rule of thumb:** Lock, do your work, release as quickly as possible!

### Quick Comparison

| What You Write | What You Get | Can Modify? | Auto-Unlock? |
|----------------|--------------|-------------|--------------||
| `state` | `Arc<Mutex<Vec>>` | ❌ No | N/A |
| `state.lock()` | `Result<MutexGuard>` | ⚠️ Must unwrap first | N/A |
| `state.lock().unwrap()` | `MutexGuard<Vec>` | ✅ Yes | ✅ Yes |

### The Magic Summary ✨

```rust
let mut team = state.lock().unwrap();
```

**Translation:**
1. "Take the shared state"
2. "Lock it (wait if someone else has it)"
3. "Give me a guard that proves I have exclusive access"
4. "Let me modify the inner Vec"
5. "When I'm done (guard drops), automatically unlock"

**You're not getting the `Vec` directly - you're getting a smart pointer that:**
- ✅ Acts like a `Vec`
- ✅ Prevents others from accessing simultaneously
- ✅ Automatically unlocks when dropped
- ✅ Enforces Rust's safety guarantees

That's why it's `MutexGuard<'_, Vec<Pokemon>>` and not just `Vec<Pokemon>`! 🦀

---

## Clone vs Move in CRUD

### Why We Need `.clone()` in CRUD Operations

#### Problem: Ownership in Rust

Rust's ownership rules prevent you from using a value in two places without cloning:

```rust
let new_pokemon = Pokemon { ... };

team.push(new_pokemon);              // ❌ Moves ownership to Vec
(StatusCode::CREATED, Json(new_pokemon))  // ❌ ERROR: Already moved!
```

#### Solution: Clone Before Pushing

```rust
let new_pokemon = Pokemon { ... };
team.push(new_pokemon.clone());      // ✅ Push a copy
(StatusCode::CREATED, Json(new_pokemon))  // ✅ Original still available
```

### Why Not References?

```rust
team.push(&new_pokemon);  // ❌ ERROR!
```

**Problem:** `Vec<Pokemon>` stores owned values, not references. `Vec<&Pokemon>` would require lifetimes and `new_pokemon` would be dropped at function end (dangling pointer).

### Why Clone in `get_all_pokemon()`?

```rust
async fn get_all_pokemon(State(state): State<SharedState>) -> Json<Vec<Pokemon>> {
    let team = state.lock().unwrap();  // team: MutexGuard<Vec<Pokemon>>
    Json(team.clone())  // ✅ Must clone!
}
```

**Reasons:**
1. `team` is `MutexGuard<Vec<Pokemon>>`, not `Vec<Pokemon>`
2. `Json()` needs **owned** `Vec<Pokemon>`
3. Can't move `Vec` out of `Mutex` (it's shared across all requests)
4. Clone creates independent copy for response

### The Immovable Vec Inside Mutex

```rust
Arc<Mutex<Vec<Pokemon>>>
           └────┬────┘
                └─ This Vec NEVER moves - it's shared state!
```

**Key principle:** The `Vec` inside the `Mutex` is the single source of truth for ALL requests. It must stay there permanently. Handlers can only:
- **Borrow** via `MutexGuard` (read/modify)
- **Clone** when returning data
- **Never move** it out

### Clone Cost

For small structs like Pokemon (~100 bytes): **negligible performance impact**. Modern computers handle this easily.

---

## Result Type & Error Handling

### Understanding `Result<T, E>`

```rust
Result<T, E>
       │  │
       │  └─ Error type (what you return when things fail)
       └─ Success type (what you return when things work)
```

### CRUD Return Type Patterns

#### CREATE - Always Succeeds
```rust
async fn create_pokemon(...) -> (StatusCode, Json<Pokemon>)
//                               └──────────┬──────────┘
//                                  Tuple: BOTH values returned
```

#### READ ONE - Might Fail (404)
```rust
async fn get_pokemon_by_id(...) -> Result<Json<Pokemon>, StatusCode>
//                                         └────┬────┘    └────┬────┘
//                                         Success (200)   Error (404)
```

### Why Result for get_pokemon_by_id?

```rust
// Pokemon exists:
Ok(Json(pokemon))         // Returns 200 + data

// Pokemon not found:
Err(StatusCode::NOT_FOUND) // Returns 404
```

**Axum automatically converts:**
- `Ok(Json(data))` → HTTP 200 with JSON body
- `Err(StatusCode::NOT_FOUND)` → HTTP 404

---

## Iterators & Closures

### What is a Closure?

Closures are anonymous inline functions:

```rust
|param| expression
 │  │       │
 │  │       └─ Body (what to do)
 │  └─ Parameter name
 └─ Closure syntax (like function args)
```

**Examples:**
```rust
|x| x * 2                    // Take x, return x*2
|a, b| a + b                 // Two parameters
|| 42                        // No parameters
|x| { println!("{}", x); x } // Multi-line body
```

### Common Iterator Methods

| Method | Purpose | Example | Returns |
|--------|---------|---------|---------|
| `.iter()` | Create iterator | `vec.iter()` | `Iterator<&T>` |
| `.map()` | Transform each item | `.map(|x| x * 2)` | New iterator |
| `.filter()` | Keep matching items | `.filter(|x| *x > 5)` | New iterator |
| `.find()` | First matching item | `.find(|x| *x == 10)` | `Option<&T>` |
| `.cloned()` | Clone each item | `.cloned()` | Owned values |
| `.collect()` | Consume into collection | `.collect::<Vec<_>>()` | `Vec<T>` |
| `.any()` | Check if any match | `.any(|x| *x == 0)` | `bool` |
| `.all()` | Check if all match | `.all(|x| *x > 0)` | `bool` |

### The get_pokemon_by_id Pattern

```rust
team.iter()
    .find(|p| p.id == id)
    .cloned()
    .map(Json)
    .ok_or(StatusCode::NOT_FOUND)
```

**Step-by-step:**

1. `.iter()` → Iterator over `&Pokemon`
2. `.find(|p| p.id == id)` → `Option<&Pokemon>` (Some if found, None if not)
3. `.cloned()` → `Option<Pokemon>` (clone to owned value)
4. `.map(Json)` → `Option<Json<Pokemon>>` (wrap in Json if Some)
5. `.ok_or(NOT_FOUND)` → `Result<Json<Pokemon>, StatusCode>` (convert None to Err)

**Flow visualization:**
```
Found:     Some(&pokemon) → Some(pokemon) → Some(Json(pokemon)) → Ok(Json(pokemon))
Not found: None           → None           → None                → Err(NOT_FOUND)
```

### Alternative Patterns

**Using `if let` (more explicit):**
```rust
if let Some(pokemon) = team.iter().find(|p| p.id == id) {
    Ok(Json(pokemon.clone()))
} else {
    Err(StatusCode::NOT_FOUND)
}
```

**Using `match` (most explicit):**
```rust
match team.iter().find(|p| p.id == id) {
    Some(pokemon) => Ok(Json(pokemon.clone())),
    None => Err(StatusCode::NOT_FOUND),
}
```

---

## Vec<T> Quick Reference

### Create & Initialize
```rust
let mut v: Vec<i32> = Vec::new();  // Empty
let v = vec![1, 2, 3];             // With values
let v = vec![0; 5];                // 5 zeros: [0,0,0,0,0]
```

### Essential Operations

| Operation | Method | Example | Notes |
|-----------|--------|---------|-------|
| **Add** | `push(x)` | `v.push(10)` | Add to end |
| | `extend(&[...])` | `v.extend(&[4,5])` | Add multiple |
| **Remove** | `pop()` | `v.pop()` | Returns `Option<T>` |
| | `remove(i)` | `v.remove(2)` | By index |
| | `retain(|x| cond)` | `v.retain(|&x| x > 5)` | Keep matching |
| | `clear()` | `v.clear()` | Remove all |
| **Access** | `v[i]` | `v[0]` | May panic |
| | `get(i)` | `v.get(2)` | Returns `Option<&T>` |
| | `first()` / `last()` | `v.last()` | Safe access |
| **Info** | `len()` | `v.len()` | Count |
| | `is_empty()` | `v.is_empty()` | Check if empty |
| **Search** | `contains(&x)` | `v.contains(&3)` | Check exists |
| | `iter().find()` | `.find(|&x| x > 5)` | Find element |
| | `iter().position()` | `.position(|&x| x == 10)` | Find index |
| **Sort** | `sort()` | `v.sort()` | Ascending |
| | `reverse()` | `v.reverse()` | Reverse |

### Iteration Patterns

```rust
// Borrow (read-only)
for x in &v { println!("{}", x); }

// Mutable borrow
for x in &mut v { *x += 1; }

// Consume (take ownership)
for x in v { /* v no longer usable */ }

// With iterator methods
let doubled: Vec<_> = v.iter().map(|x| x * 2).collect();
let filtered: Vec<_> = v.iter().filter(|&&x| x > 3).cloned().collect();
```

### Safe ID Generation Pattern

```rust
// ❌ Unsafe - panics if empty
let new_id = team.last().unwrap().id + 1;

// ✅ Safe - handles empty Vec
let new_id = team.last().map(|p| p.id + 1).unwrap_or(1);

// ✅ Also safe - using if let
let new_id = if let Some(last) = team.last() {
    last.id + 1
} else {
    1
};
```

---

## Mutable vs Immutable Methods

### The `_mut()` Pattern

In Rust, methods that return mutable references follow the `method_mut()` naming convention:

```
method()      → Returns immutable reference/iterator (read-only)
method_mut()  → Returns mutable reference/iterator (read-write)
```

### Common `_mut()` Variants

#### Iterators
```rust
let mut vec = vec![1, 2, 3, 4, 5];

// Immutable - can only read
for x in vec.iter() {
    println!("{}", x);
}

// Mutable - can modify
for x in vec.iter_mut() {
    *x *= 2;
}
```

#### Access Methods
```rust
let mut vec = vec![1, 2, 3];

// Read-only access
if let Some(first) = vec.first() {
    println!("{}", first);
}

// Mutable access
if let Some(first) = vec.first_mut() {
    *first = 10;
}
```

### Quick Reference

| Category | Immutable | Mutable | Use Case |
|----------|-----------|---------|----------|
| **Iterator** | `.iter()` | `.iter_mut()` | Loop over items |
| **Access** | `.get(i)` | `.get_mut(i)` | Get by index |
| | `.first()` | `.first_mut()` | First element |
| | `.last()` | `.last_mut()` | Last element |
| **Slicing** | `.split()` | `.split_mut()` | Split into parts |
| | `.chunks(n)` | `.chunks_mut(n)` | Fixed-size chunks |
| | `.split_at(i)` | `.split_at_mut(i)` | Split at index |
| **HashMap** | `.get(&k)` | `.get_mut(&k)` | Get value by key |
| | `.values()` | `.values_mut()` | Iterate values |
| **String** | `.as_bytes()` | `.as_bytes_mut()` | Byte slice |
| **Option** | `.as_ref()` | `.as_mut()` | Convert to ref |
| **RefCell** | `.borrow()` | `.borrow_mut()` | Interior mutability |

### UPDATE Operation Example

```rust
// ❌ Wrong - immutable iterator
let team = state.lock().unwrap();
if let Some(pokemon) = team.iter().find(|p| p.id == id) {
    pokemon.level = 50;  // ERROR: Can't mutate!
}

// ✅ Correct - mutable iterator
let mut team = state.lock().unwrap();  // Must be mut
if let Some(pokemon) = team.iter_mut().find(|p| p.id == id) {
    pokemon.level = 50;  // Works!
}
```

### Rust's Borrowing Rules

```
🔒 Immutable borrows: Multiple allowed simultaneously
🔓 Mutable borrow: Only ONE at a time
❌ Can't mix: No immutable + mutable at same time
```

**Mental model:**
- Use regular method when you just want to **look**
- Use `_mut()` method when you want to **change**

---

## CRUD Function Design

### The Complete Pattern

For each CRUD operation, follow these rules:

### 1. CREATE - Add New Item

```rust
async fn create_resource(
    State(state): State<SharedState>,     // ✅ Need state to add to
    Json(payload): Json<CreateResource>,  // ✅ Need data from client
) -> (StatusCode, Json<Resource>)         // ✅ Return 201 + new item
```

**Rules:**
- ✅ Takes `State` (to modify collection)
- ✅ Takes `Json(CreateDTO)` (client doesn't send ID)
- ✅ Returns `201 CREATED` + the new item (with generated ID)
- ✅ Generate ID server-side, never trust client

---

### 2. READ ALL - Get Collection

```rust
async fn get_all_resources(
    State(state): State<SharedState>,  // ✅ Need state to read from
) -> Json<Vec<Resource>>               // ✅ Return 200 + array
```

**Rules:**
- ✅ Takes `State` (to read collection)
- ❌ No `Path` or `Json` needed
- ✅ Returns `200 OK` + array (even if empty)

---

### 3. READ ONE - Get Single Item

```rust
async fn get_resource_by_id(
    State(state): State<SharedState>,      // ✅ Need state to search
    Path(id): Path<u32>,                   // ✅ Need ID to find
) -> Result<Json<Resource>, StatusCode>    // ✅ Return 200 or 404
```

**Rules:**
- ✅ Takes `State` + `Path(id)`
- ✅ Returns `Result<>` (might not find it)
- ✅ `200 OK` if found, `404 NOT FOUND` if missing

---

### 4. UPDATE - Modify Existing Item

```rust
async fn update_resource(
    State(state): State<SharedState>,      // ✅ Need state to modify
    Path(id): Path<u32>,                   // ✅ Need ID to find
    Json(payload): Json<UpdateResource>,   // ✅ Need new data
) -> Result<Json<Resource>, StatusCode>    // ✅ Return 200 or 404
```

**Rules:**
- ✅ Takes `State` + `Path(id)` + `Json(UpdateDTO)`
- ✅ Use `Option<T>` fields in DTO (partial updates)
- ✅ Returns `200 OK` + updated item, or `404 NOT FOUND`

---

### 5. DELETE - Remove Item

```rust
async fn delete_resource(
    State(state): State<SharedState>,  // ✅ Need state to modify
    Path(id): Path<u32>,               // ✅ Need ID to find
) -> StatusCode                        // ✅ Return 204 or 404
```

**Rules:**
- ✅ Takes `State` + `Path(id)`
- ❌ No `Json` needed (nothing to receive)
- ✅ Returns `204 NO CONTENT` if deleted, `404 NOT FOUND` if not exists
- ❌ Don't return the deleted item (waste of bandwidth)

---

### Quick Decision Tree 🌳

```
Do I need to modify data?
├─ YES → Take State(state)
└─ NO  → (rare for CRUD, maybe health check)

Am I working on ONE specific item?
├─ YES → Take Path(id)
└─ NO  → Working on whole collection

Am I receiving data from client?
├─ YES → Take Json(payload)
└─ NO  → Just reading/deleting

Might the operation fail?
├─ YES → Return Result<T, StatusCode>
└─ NO  → Return direct value or tuple
```

---

### Pattern Matching Table

| Operation | State? | Path(id)? | Json? | Returns |
|-----------|--------|-----------|-------|---------|
| CREATE | ✅ | ❌ | ✅ CreateDTO | `(201, Json<T>)` |
| READ ALL | ✅ | ❌ | ❌ | `Json<Vec<T>>` |
| READ ONE | ✅ | ✅ | ❌ | `Result<Json<T>, StatusCode>` |
| UPDATE | ✅ | ✅ | ✅ UpdateDTO | `Result<Json<T>, StatusCode>` |
| DELETE | ✅ | ✅ | ❌ | `StatusCode` |

---

### Common Mistakes to Avoid ❌

#### ❌ Returning item on DELETE
```rust
// Don't do this:
async fn delete_resource(...) -> Json<Resource> { }

// Do this:
async fn delete_resource(...) -> StatusCode { }
```

#### ❌ Accepting ID in CREATE
```rust
// Don't do this:
#[derive(Deserialize)]
struct CreatePokemon {
    id: u32,  // ❌ Server should generate this!
}

// Do this:
#[derive(Deserialize)]
struct CreatePokemon {
    // No id field - server generates it
}
```

#### ❌ Required fields in UPDATE DTO
```rust
// Don't do this:
struct UpdatePokemon {
    name: String,  // ❌ Forces client to send all fields
}

// Do this:
struct UpdatePokemon {
    name: Option<String>,  // ✅ Allows partial updates
}
```

---

### HTTP Status Code Reference 🚦

| Code | When to Use |
|------|-------------|
| `200 OK` | Successful read/update |
| `201 CREATED` | Successful creation |
| `204 NO CONTENT` | Successful deletion (no body) |
| `400 BAD REQUEST` | Client sent invalid data |
| `404 NOT FOUND` | Item doesn't exist |
| `500 INTERNAL SERVER ERROR` | Something broke on server |

---

### The Ultimate Checklist ✅

For each CRUD function, ask:

1. **Do I modify data?** → Need `State`
2. **Do I work on one item?** → Need `Path(id)`
3. **Do I receive data?** → Need `Json(DTO)`
4. **Can this fail?** → Return `Result<>`
5. **What status code?** → See table above

---

## Additional Resources 📖

### Official Documentation
- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust fundamentals
- [Axum Documentation](https://docs.rs/axum/latest/axum/) - Framework reference
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial) - Async runtime
- [Serde Guide](https://serde.rs/) - JSON serialization

### Community
- [Rust Discord](https://discord.gg/rust-lang) - Ask for help
- [r/rust](https://reddit.com/r/rust) - Reddit community
- [Rust Users Forum](https://users.rust-lang.org/) - Discussion forum

---

## Quick Reference Commands 🚀

```bash
# Check if code compiles (fast)
cargo check

# Build and run server
cargo run

# Build for release (optimized)
cargo build --release

# Format code
cargo fmt

# Run linter
cargo clippy

# Clean build artifacts
cargo clean

# Add a dependency
cargo add <package_name>
```

---

**Last Updated:** 2025-10-30
**Latest Additions:** Clone vs Move, Result Types, Iterators & Closures, Vec Quick Reference, Mutable Methods

**Remember:** This is a living document. Add your own notes and discoveries as you learn!
