use bevy::prelude::*;

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use super::net::{ConnectionInfo, CreateMessages};

// game stuff
#[derive(Properties, Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message: String,
    pub from: String,
    pub ordinal: u8,
}

impl Message {
    pub fn new(message: &str, from: &str, ordinal: u8) -> Message {
        Message {
            message: message.to_string(),
            from: from.to_string(),
            ordinal: ordinal,
        }
    }
}

pub struct Cube;

pub fn build(app: &mut AppBuilder) {
    app.add_startup_system(setup.system())
        .add_system(move_cube_system.system())
        .add_system(add_message_system.system())
        .add_system(message_compact_system.system());
}

fn setup(
    mut commands: Commands,
    ci: Res<ConnectionInfo>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let server = ci.is_server();

    // add entities to the world
    commands
        // plane
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb(0.1, 0.2, 0.1).into()),
            ..Default::default()
        })
        // cube
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.5, 0.4, 0.3).into()),
            translation: Translation::new(0.0, 1.0, 0.0),
            ..Default::default()
        })
        .with(Cube)
        // light
        .spawn(LightComponents {
            translation: Translation::new(4.0, 8.0, 4.0),
            ..Default::default()
        })
        // camera
        .spawn(Camera3dComponents {
            transform: Transform::new_sync_disabled(Mat4::face_toward(
                Vec3::new(-2.0, 6.0, 12.0),
                Vec3::new(-2.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        });
    if server {
        commands.spawn((Message {
            message: "starting".to_string(),
            from: "server".to_string(),
            ordinal: 0,
        },));
    }
}

fn move_cube_system(
    time: Res<Time>,
    ci: Res<ConnectionInfo>,
    keyboard_input: Res<Input<KeyCode>>,
    mut cubes: Query<(&Cube, &mut Translation)>,
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

    let delta = Vec3::new(x, 0f32, z) * speed * time.delta_seconds;

    for (_cube, mut tx) in &mut cubes.iter() {
        let mut pos = tx.0 + delta;
        pos.set_x(pos.x().max(-4.0).min(4.0));
        pos.set_z(pos.z().max(-4.0).min(4.0));
        tx.0 = pos;
    }
}

fn message_compact_system(mut messages: Query<&mut Message>) {
    let mut i = messages.iter();
    let mut sorted_displays: Vec<Mut<Message>> = i.into_iter().collect();

    sorted_displays.sort_by(|a, b| a.ordinal.cmp(&b.ordinal));

    let mut next = 0;
    let mut needs_compact = false;

    for message in &sorted_displays {
        if message.ordinal != next {
            needs_compact = true;
        }
        next = next + 1;
    }

    if needs_compact {
        let mut next = 0;
        for message in &mut sorted_displays {
            message.ordinal = next;
            next = next + 1;
        }
    }
}

fn add_message_system(
    mut commands: Commands,
    ci: Res<ConnectionInfo>,
    mut network_create_messages: ResMut<CreateMessages>,
    mut interaction_query: Query<(&Button, Mutated<Interaction>)>,
) {
    for (_button, interaction) in &mut interaction_query.iter() {
        if let Interaction::Clicked = *interaction {
            if ci.is_server() {
                // immediately create the message
                commands.spawn((Message::new(&random_message(), "server", 255),));
            } else {
                // schedule the message to be sent to the server
                network_create_messages.messages.push(random_message());
            }
        }
    }
}

fn random_message() -> String {
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

    return msgs.choose(&mut rand::thread_rng()).unwrap().to_string();
}
