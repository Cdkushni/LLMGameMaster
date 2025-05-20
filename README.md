# LLM Game Master

A dynamic game master built with Rust, Actix-Web, Yew, and SQLite. It maintains a world state, allows player actions, and generates story events with branching choices using an LLM (Grok API) to write and manage a game world story.

## Features
- World state management (locations, factions, player, NPCs).
- Player actions: Move, Help, Fight.
- LLM-generated story events with branching paths.
- Web UI for interacting with the game world.

## Setup
### Prerequisites
- Rust (latest stable)
- Trunk (`cargo install trunk`)
- SQLite
- Grok API key (set as `GROK_API_KEY`)

### Installation
1. Clone the repo:
   ```bash
   git clone https://github.com/your-username/sci-fi-gm.git
   cd sci-fi-gm