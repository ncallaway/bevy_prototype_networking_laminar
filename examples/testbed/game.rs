use bevy::prelude::*;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use super::net::{ConnectionInfo, CreateNotes};

// game stuff
#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub note: String,
    pub from: String,
    pub ordinal: u8,
}

impl Note {
    pub fn new(note: &str, from: &str, ordinal: u8) -> Note {
        Note {
            note: note.to_string(),
            from: from.to_string(),
            ordinal,
        }
    }
}

pub struct Cube;

pub fn build(app: &mut AppBuilder) {
    app.add_startup_system(setup.system())
        .add_system(move_cube_system.system())
        .add_system(add_note_system.system())
        .add_system_to_stage(super::ui::stages::DOMAIN_SYNC, note_compact_system.system());
}

fn setup(
    commands: &mut Commands,
    ci: Res<ConnectionInfo>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let server = ci.is_server();

    // add entities to the world
    commands
        // plane
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb(0.1, 0.2, 0.1).into()),
            ..Default::default()
        })
        // cube
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.5, 0.4, 0.3).into()),
            transform: Transform {
                translation: Vec3::new(0.0, 1.0, 0.0),
                rotation: Default::default(),
                scale: Default::default(),
            },
            ..Default::default()
        })
        .with(Cube)
        // light
        .spawn(LightBundle {
            transform: Transform {
                translation: Vec3::new(4.0, 8.0, 4.0),
                rotation: Default::default(),
                scale: Default::default(),
            },
            ..Default::default()
        })
        // camera
        .spawn(Camera3dBundle {
            transform: Transform::from_matrix(Mat4::face_toward(
                Vec3::new(-2.0, 6.0, 12.0),
                Vec3::new(-2.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        });
    if server {
        commands.spawn((Note {
            note: "starting".to_string(),
            from: "server".to_string(),
            ordinal: 0,
        },));
    }
}

fn move_cube_system(
    time: Res<Time>,
    ci: Res<ConnectionInfo>,
    keyboard_input: Res<Input<KeyCode>>,
    mut cubes: Query<(&Cube, &mut Transform)>,
) {
    if ci.is_client() {
        return;
    }

    let speed = 5.0;
    let mut x = 0f32;
    let mut z = 0f32;
    if keyboard_input.pressed(KeyCode::A) {
        x -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::D) {
        x += 1.0;
    }

    if keyboard_input.pressed(KeyCode::W) {
        z -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::S) {
        z += 1.0;
    }

    let delta = Vec3::new(x, 0f32, z) * speed * time.delta_seconds();

    for (_cube, mut tx) in &mut cubes.iter_mut() {
        let mut pos = tx.translation + delta;
        pos.x = pos.x.max(-4.0).min(4.0);
        pos.z = pos.z.max(-4.0).min(4.0);
        tx.translation = pos;
    }
}

fn note_compact_system(mut notes: Query<&mut Note>) {
    let i = notes.iter_mut();
    let mut sorted_displays: Vec<Mut<Note>> = i.into_iter().collect();

    sorted_displays.sort_by(|a, b| a.ordinal.cmp(&b.ordinal));

    let mut needs_compact = false;

    for (next, note) in sorted_displays.iter().enumerate() {
        if note.ordinal != next as u8 {
            needs_compact = true;
        }
    }

    if needs_compact {
        for (next, note) in &mut sorted_displays.iter_mut().enumerate() {
            note.ordinal = next as u8;
        }
    }
}

fn add_note_system(
    commands: &mut Commands,
    ci: Res<ConnectionInfo>,
    mut network_create_notes: ResMut<CreateNotes>,
    interaction_query: Query<(&Button, &Interaction), Mutated<Interaction>>,
) {
    for (_button, interaction) in &mut interaction_query.iter() {
        if let Interaction::Clicked = *interaction {
            if ci.is_server() {
                // immediately create the note
                commands.spawn((Note::new(&random_phrase(), "server", 255),));
            } else {
                // schedule the note to be sent to the server
                network_create_notes.notes.push(random_phrase());
            }
        }
    }
}

fn random_phrase() -> String {
    let msgs = [
        "Lorem ipsum dolor sit amet",
        "consectetur adipiscing elit",
        "Proin vel eros dolor",
        "Cras luctus vehicula ex",
        "Proin vel eros dolor",
        "at dapibus massa viverra id",
        "Vestibulum nec tempor lacus",
        "eget lobortis ligula",
        "Sed vel gravida neque",
        "ac sollicitudin purus",
        "Aenean aliquet odio quis nulla varius efficitur",
        "Phasellus vitae nibh leo",
        "Maecenas lobortis porttitor consectetur",
        "Sed congue, ex a blandit congue",
        "erat erat ullamcorper orci",
        "vitae euismod eros lacus ut eros",
        "Ut molestie metus leo",
        "eget posuere tellus maximus a",
        "Nulla porttitor faucibus ullamcorper",
        "Phasellus feugiat felis at odio consectetur lacinia. Nullam fermentum malesuada consequat",
    ];

    msgs.choose(&mut rand::thread_rng()).unwrap().to_string()
}
