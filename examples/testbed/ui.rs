use bevy::prelude::*;

use super::game::Note;

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

struct NoteDisplay {
    ordinal: u8,
}

struct NoteContainer;

impl FromResources for ButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.02, 0.02, 0.02).into()),
            hovered: materials.add(Color::rgb(0.05, 0.05, 0.05).into()),
            pressed: materials.add(Color::rgb(0.1, 0.5, 0.1).into()),
        }
    }
}

pub mod stages {
    // const STAGE_UI_BEFORE: &str = "ui_before";
    // const STAGE_UI_AFTER: &str = "ui_after";

    pub const USER_EVENTS: &str = "user_events";
    pub const DOMAIN_EVENTS: &str = "domain_events";

    pub const DOMAIN_SYNC: &str = "domain_sync";
    pub const VISUAL_SYNC: &str = "visual_sync";
}

pub fn build(app: &mut AppBuilder) {
    app.init_resource::<ButtonMaterials>()
        .init_resource::<Handle<Font>>()
        .add_startup_system(setup_ui.system())
        .add_stage_before(stage::UPDATE, stages::USER_EVENTS, SystemStage::parallel())
        .add_stage_after(
            stage::UPDATE,
            stages::DOMAIN_EVENTS,
            SystemStage::parallel(),
        )
        .add_stage_after(
            stages::DOMAIN_EVENTS,
            stages::DOMAIN_SYNC,
            SystemStage::parallel(),
        )
        .add_stage_after(
            stages::DOMAIN_SYNC,
            stages::VISUAL_SYNC,
            SystemStage::parallel(),
        )
        .add_system_to_stage(stages::USER_EVENTS, button_hover_system.system())
        .add_system(note_display_sync_system.system())
        .add_system_to_stage(stages::VISUAL_SYNC, note_display_ordering_system.system());
}

fn setup_ui(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut font_handle: ResMut<Handle<Font>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    button_materials: Res<ButtonMaterials>,
) {
    *font_handle = asset_server.load("fonts/FiraMono-Medium.ttf");

    let background = Color::rgba(0.0, 0.0, 0.0, 0.9);

    commands
        // 2d camera
        .spawn(CameraUiBundle::default())
        // root sidebar
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(325.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            material: materials.add(background.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                // button
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Px(45.0)),
                        // center button
                        margin: Rect::all(Val::Px(0.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // // vertically center child text
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    material: button_materials.normal.clone(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    // button label
                    parent.spawn(TextBundle {
                        text: Text {
                            value: "Send a note".to_string(),
                            font: font_handle.clone(),
                            style: TextStyle {
                                font_size: 12.0,
                                color: Color::rgb(0.8, 0.8, 0.8),
                                ..Default::default()
                            },
                        },
                        ..Default::default()
                    });
                })
                // notes box
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Auto),
                        justify_content: JustifyContent::FlexStart,
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                    material: materials.add(background.into()),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Auto),
                                justify_content: JustifyContent::FlexStart,
                                flex_direction: FlexDirection::Column,
                                ..Default::default()
                            },
                            material: materials.add(background.into()),
                            ..Default::default()
                        })
                        .with(NoteContainer)
                        .spawn(TextBundle {
                            style: Style {
                                align_self: AlignSelf::Center,
                                ..Default::default()
                            },
                            text: Text {
                                value: "NOTES ".to_string(),
                                font: font_handle.clone(),
                                style: TextStyle {
                                    font_size: 24.0,
                                    color: Color::WHITE,
                                    ..Default::default()
                                },
                            },
                            ..Default::default()
                        });
                });
        });
}

fn button_hover_system(
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Button, &Interaction, &mut Handle<ColorMaterial>),
        Mutated<Interaction>,
    >,
) {
    for (_button, interaction, mut material) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *material = button_materials.pressed.clone();
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}

fn note_display_sync_system(
    mut commands: &mut Commands,
    font_handle: Res<Handle<Font>>,
    notes: Query<&Note>,
    mut display_containers: Query<(Entity, &NoteContainer)>,
    mut note_displays: Query<(Entity, &mut NoteDisplay, &mut Text)>,
) {
    let mut display_iter = note_displays.iter_mut();
    for note in &mut notes.iter() {
        let has_display = display_iter.next();

        match has_display {
            Some((_, mut display, mut text)) => {
                // oh no! A string allocation in a system loop! That's not great. I don't care
                // about the performance of this, but please don't copy this somewhere real.
                let desired_text = format!("[{}] {}", note.from, note.note);
                if text.value != desired_text || display.ordinal != note.ordinal {
                    text.value = format!("[{}] {}", note.from, note.note);
                    display.ordinal = note.ordinal;
                }
            }
            None => spawn_note_display(
                &mut commands,
                font_handle.clone(),
                &mut display_containers,
                format!("[{}] {}", note.from, note.note),
                note.ordinal,
            ),
        }
    }

    // remove the remaining entities
    for (e, _, _) in display_iter {
        commands.despawn(e);
    }
}

fn note_display_ordering_system(
    mut containers: Query<(Entity, &NoteContainer, &mut Children)>,
    note_display: Query<&NoteDisplay>,
) {
    // get the children of the note container
    let container_borrow = containers.iter_mut();
    let mut container_children = match container_borrow.into_iter().next() {
        Some((_, _, c)) => c,
        None => return,
    };

    let mut last = None;
    let mut needs_reorder = false;

    for child in container_children.iter() {
        if let Ok(note) = note_display.get(*child) {
            if let Some(prior) = last {
                if note.ordinal > prior {
                    needs_reorder = true;
                }
            }

            last = Some(note.ordinal);
        }
    }

    if needs_reorder {
        let mut vec: Vec<Entity> = container_children.iter().copied().collect();
        vec.sort_by(|a, b| {
            let a_ordinal = note_display.get(*a).unwrap().ordinal;
            let b_ordinal = note_display.get(*b).unwrap().ordinal;
            b_ordinal.cmp(&a_ordinal)
        });
        *container_children = Children::with(&vec[..]);
    }
}

fn spawn_note_display(
    commands: &mut Commands,
    font_handle: Handle<Font>,
    display_containers: &mut Query<(Entity, &NoteContainer)>,
    value: String,
    ordinal: u8,
) {
    // have to have a notes container, or we die, sorry.
    let (container_entity, _) = display_containers.iter().into_iter().next().unwrap();
    spawn_note_display_with_entity(commands, &font_handle, container_entity, value, ordinal);
}

fn spawn_note_display_with_entity(
    commands: &mut Commands,
    font_handle: &Handle<Font>,
    container_entity: Entity,
    value: String,
    ordinal: u8,
) {
    let md = NoteDisplay { ordinal };

    commands
        .spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexStart,
                ..Default::default()
            },
            text: Text {
                value,
                font: font_handle.clone(),
                style: TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..Default::default()
                },
            },
            ..Default::default()
        })
        .with(md)
        .with(Parent(container_entity));
}
