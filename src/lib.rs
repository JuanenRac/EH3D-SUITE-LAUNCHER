// EH3D-SUITE-LAUNCHER - the hub of the EH3D Fabrication Suite.
// Copyright (C) 2026 JuanenRac (Electro Hobby 3D). GPL-3.0-or-later.
#![forbid(unsafe_code)]
//! Finds the programs of the suite by their manifests, starts them, keeps the list of recent projects and reads and writes the project file. With the `gui` feature it also has the window (`window` holds its state, `gui` draws it).

pub mod browse;
#[cfg(feature = "gui")]
pub mod gui;
pub mod lang;
pub mod launch;
pub mod limit;
pub mod project;
pub mod recent;
pub mod registry;
pub mod settings;
pub mod shot;
pub mod window;
