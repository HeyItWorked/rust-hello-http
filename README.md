# Pokémon Team Manager API 🦀

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/) [![Axum](https://img.shields.io/badge/axum-0.8.6-blue.svg)](https://github.com/tokio-rs/axum) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A RESTful CRUD API for managing a Pokémon team, built with Rust and Axum. This project demonstrates core concepts of web development in Rust including async/await, shared state management, and REST API design.

**Note:** This is an educational project designed to showcase Rust web development fundamentals.

---

## ✨ Features

- **Full CRUD Operations** - Create, Read, Update, and Delete Pokémon
- **RESTful API Design** - Follows REST conventions with proper HTTP methods and status codes
- **Thread-Safe State Management** - Uses `Arc<Mutex<T>>` for concurrent access
- **Type-Safe JSON Handling** - Leverages Serde for serialization/deserialization
- **In-Memory Storage** - Simple Vec-based storage (no database required)

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.70 or higher ([Install Rust](https://rustup.rs/))

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/rust-hello-http.git
cd rust-hello-http

# Build and run
cargo run
```

The server will start on `http://localhost:3000`

---

## 📡 API Endpoints

### Create a Pokémon
```bash
POST /pokemon
Content-Type: application/json

{
  "name": "Pikachu",
  "poke_type": "Electric",
  "level": 25
}

# Response: 201 Created
{
  "id": 1,
  "name": "Pikachu",
  "poke_type": "Electric",
  "level": 25
}
```

### Get All Pokémon
```bash
GET /pokemon

# Response: 200 OK
[
  {
    "id": 1,
    "name": "Pikachu",
    "poke_type": "Electric",
    "level": 25
  }
]
```

### Get Pokémon by ID
```bash
GET /pokemon/{id}

# Response: 200 OK (if found)
{
  "id": 1,
  "name": "Pikachu",
  "poke_type": "Electric",
  "level": 25
}

# Response: 404 Not Found (if not found)
```

### Update a Pokémon
```bash
PUT /pokemon/{id}
Content-Type: application/json

{
  "level": 30
}

# Response: 200 OK
{
  "id": 1,
  "name": "Pikachu",
  "poke_type": "Electric",
  "level": 30
}

# Note: All fields are optional for partial updates
```

### Delete a Pokémon
```bash
DELETE /pokemon/{id}

# Response: 204 No Content (if deleted)
# Response: 404 Not Found (if not found)
```

---

## 🧪 Testing the API

### Using cURL

```bash
# Create a Pokémon
curl -X POST http://localhost:3000/pokemon \
  -H "Content-Type: application/json" \
  -d '{"name":"Pikachu","poke_type":"Electric","level":25}'

# Get all Pokémon
curl http://localhost:3000/pokemon

# Get one Pokémon
curl http://localhost:3000/pokemon/1

# Update a Pokémon
curl -X PUT http://localhost:3000/pokemon/1 \
  -H "Content-Type: application/json" \
  -d '{"level":30}'

# Delete a Pokémon
curl -X DELETE http://localhost:3000/pokemon/1
```

### Using Postman

Import these endpoints into Postman:

| Method | URL | Body |
|--------|-----|------|
| POST | `http://localhost:3000/pokemon` | `{"name":"Pikachu","poke_type":"Electric","level":25}` |
| GET | `http://localhost:3000/pokemon` | - |
| GET | `http://localhost:3000/pokemon/1` | - |
| PUT | `http://localhost:3000/pokemon/1` | `{"level":30}` |
| DELETE | `http://localhost:3000/pokemon/1` | - |

---

## 🏗️ Project Structure

```
rust-hello-http/
├── Cargo.toml           # Dependencies and project metadata
├── Cargo.lock           # Dependency lock file
├── README.md            # This file
└── src/
    ├── main.rs          # Server setup and route handlers
    └── models.rs        # Data models (Pokemon, CreatePokemon, UpdatePokemon)
```

---

## 🛠️ Technical Stack

- **[Axum 0.8](https://github.com/tokio-rs/axum)** - Web framework
- **[Tokio](https://tokio.rs/)** - Async runtime
- **[Serde](https://serde.rs/)** - JSON serialization/deserialization
- **Rust Standard Library** - `Arc<Mutex<T>>` for shared state

### Key Design Patterns

- **DTOs (Data Transfer Objects)** - Separate structs for create/update operations
- **Shared State** - `Arc<Mutex<Vec<Pokemon>>>` for thread-safe concurrent access
- **Extractors** - Type-safe parameter extraction (`State`, `Json`, `Path`)
- **Result Types** - Proper error handling with `Result<T, E>`

---

## 🎓 What This Project Demonstrates

### Rust Concepts
- Ownership and borrowing
- `Option<T>` and `Result<T, E>` types
- Pattern matching (`if let`, `match`)
- Closures and iterators
- Shared state with `Arc<Mutex<T>>`

### Web Development
- RESTful API design
- HTTP methods and status codes
- JSON serialization/deserialization
- Request/response handling
- Path parameters

### Axum Framework
- Router configuration
- Handler functions
- Extractors (`State`, `Json`, `Path`)
- Shared application state

---

## 🐛 Troubleshooting

### Port Already in Use
```bash
# Find and kill the process using port 3000
lsof -i :3000
kill -9 <PID>

# Or change the port in main.rs
```

### Data Disappears on Restart
This is expected behavior. The API uses in-memory storage. Data is lost when the server stops. For persistence, consider adding a database or file-based storage.

---

## 🚀 Future Enhancements

- [ ] Add input validation
- [ ] Implement error response with JSON
- [ ] Add persistence (database or file storage)
- [ ] Add search/filter endpoints
- [ ] Add pagination for large datasets
- [ ] Add tests (unit and integration)
- [ ] Add logging middleware
- [ ] Add CORS support

---

## 🤝 Contributing

Contributions are welcome! Feel free to:
- Report bugs
- Suggest features
- Submit pull requests

---

## 📄 License

This project is licensed under the MIT License.

```
MIT License

Copyright (c) 2025

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## 📚 Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust fundamentals
- [Axum Documentation](https://docs.rs/axum/latest/axum/) - Framework reference
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial) - Async runtime guide
- [Serde Guide](https://serde.rs/) - JSON serialization

---

**Built with Rust 🦀**
