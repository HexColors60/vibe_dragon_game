# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A 3D dinosaur hunting game built with Rust and the Bevy game engine. The player drives a pickup truck with a mounted machine gun, hunting dinosaurs in a Jurassic-style forest environment.

## Build and Run Commands

```bash
# Build the project
cargo build

# Run the game
cargo run

# Run with release optimizations (recommended for gameplay)
cargo run --release

# Run tests
cargo test

# Check code without building
cargo check
```

## Architecture

The project uses Bevy's Entity Component System (ECS) architecture. Key planned modules:

- **main.rs** - Entry point, app initialization, plugin registration
- **camera.rs** - Third-person camera following the pickup truck
- **input.rs** - WASD/arrow key movement + mouse aiming
- **vehicle.rs** - Pickup truck physics and controls
- **dino.rs** - Dinosaur entities, species types, health/damage systems
- **weapon.rs** - Machine gun firing, raycasting/ballistic hit detection
- **ui.rs** - HUD with crosshair, health bars, scoring
- **ai.rs** - Dinosaur state machine (Idle/Ram/Flee/Dead)

## Game Design Reference

The full design specification is in `doc/design01.txt` (in Chinese). Key requirements:

- **Vehicle**: Pickup truck with roof-mounted machine gun, WASD movement
- **Weapon**: Mouse-aimed, left-click to fire
- **Dinosaurs**: At least 3 species (Triceratops, Velociraptor, Brachiosaurus) with different health/speed
- **Hit detection**: Critical body parts (head, heart) for bonus damage
- **Environment**: Jurassic forest with hills, rocks, rivers, volcanoes, flying pterosaurs
- **Feedback**: Blood particle effects, sound effects, scoring system

## Dependencies

Key crates (to be added to Cargo.toml):
- `bevy` - Core game engine
- `bevy_rapier3d` - 3D physics and collision detection
- Additional crates for audio, particles as needed

## Current State

This is a new project in the design phase. The Cargo.toml and src/ directory have not been created yet. Implementation should follow the design document in `doc/design01.txt`.
