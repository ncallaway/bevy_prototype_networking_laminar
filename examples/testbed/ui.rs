use bevy::prelude::*;

use smallvec::SmallVec;

use super::game::Note;

const UI_BACKGROUND: Color = Color::rgba(0.0, 0.0, 0.0, 0.9);

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
        .add_stage_before(stage::UPDATE, stages::USER_EVENTS)
        .add_stage_after(stage::UPDATE, stages::DOMAIN_EVENTS)
        .add_stage_after(stages::DOMAIN_EVENTS, stages::DOMAIN_SYNC)
        .add_stage_after(stages::DOMAIN_SYNC, stages::VISUAL_SYNC)
        .add_system_to_stage(stages::USER_EVENTS, button_hover_system.system())
        .add_system(note_display_sync_system.system())
        .add_system_to_stage(stages::VISUAL_SYNC, note_display_ordering_system.system());
}

fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut font_handle: ResMut<Handle<Font>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    button_materials: Res<ButtonMaterials>,
) {
    *font_handle = asset_server
        .load("assets/fonts/FiraMono-Medium.ttf")
        .unwrap();

    commands
        // 2d camera
        .spawn(UiCameraComponents::default())
        // root sidebar
        .spawn(NodeComponents {
            style: Style {
                size: Size::new(Val::Px(325.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            material: materials.add(UI_BACKGROUND.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                // button
                .spawn(ButtonComponents {
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
                    material: button_materials.normal,
                    ..Default::default()
                })
                .with_children(|parent| {
                    // button label
                    parent.spawn(TextComponents {
                        text: Text {
                            value: "Send a note".to_string(),
                            font: *font_handle,
                            style: TextStyle {
                                font_size: 12.0,
                                color: Color::rgb(0.8, 0.8, 0.8),
                            },
                        },
                        ..Default::default()
                    });
                })
                // notes box
                .spawn(NodeComponents {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Auto),
                        justify_content: JustifyContent::FlexStart,
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                    material: materials.add(UI_BACKGROUND.into()),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeComponents {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Auto),
                                justify_content: JustifyContent::FlexStart,
                                flex_direction: FlexDirection::Column,
                                ..Default::default()
                            },
                            material: materials.add(UI_BACKGROUND.into()),
                            ..Default::default()
                        })
                        .with(NoteContainer)
                        .spawn(TextComponents {
                            style: Style {
                                align_self: AlignSelf::Center,
                                ..Default::default()
                            },
                            text: Text {
                                value: "NOTES ".to_string(),
                                font: *font_handle,
                                style: TextStyle {
                                    font_size: 24.0,
                                    color: Color::WHITE,
                                },
                            },
                            ..Default::default()
                        });
                });
        });
}

fn button_hover_system(
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<(&Button, Mutated<Interaction>, &mut Handle<ColorMaterial>)>,
) {
    for (_, interaction, mut material) in &mut interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => {
                *material = button_materials.pressed;
            }
            Interaction::Hovered => {
                *material = button_materials.hovered;
            }
            Interaction::None => {
                *material = button_materials.normal;
            }
        }
    }
}

fn note_display_sync_system(
    mut commands: Commands,
    font_handle: Res<Handle<Font>>,
    mut notes: Query<&Note>,
    mut display_containers: Query<(Entity, &NoteContainer)>,
    mut note_displays: Query<(Entity, &mut NoteDisplay, &mut Text)>,
) {
    let mut display_borrow = note_displays.iter();
    let mut display_iter = display_borrow.into_iter();

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
                *font_handle,
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
    mut note_display: Query<(Entity, &mut NoteDisplay)>,
) {
    // get the children of the note container
    let mut container_borrow = containers.iter();
    let mut container_children = match container_borrow.into_iter().next() {
        Some((_, _, c)) => c,
        None => return,
    };

    // (_, _, mut container_children) = ?;

    let mut last = None;
    let mut needs_reorder = false;

    for child in container_children.iter() {
        let d = note_display.get::<NoteDisplay>(*child).unwrap();

        if let Some(prior) = last {
            if d.ordinal > prior {
                needs_reorder = true;
            }
        }

        last = Some(d.ordinal);
    }

    if needs_reorder {
        let mut display_borrow = note_display.iter();
        let mut sorted_displays: Vec<(Entity, Mut<NoteDisplay>)> =
            display_borrow.into_iter().collect();

        sorted_displays.sort_by(|(_, a), (_, b)| b.ordinal.cmp(&a.ordinal));

        let sorted_entities: Vec<Entity> = sorted_displays.iter().map(|(e, _)| *e).collect();
        container_children.0 = SmallVec::from_slice(&sorted_entities[..]);
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
        .spawn(TextComponents {
            style: Style {
                align_self: AlignSelf::FlexStart,
                ..Default::default()
            },
            text: Text {
                value,
                font: *font_handle,
                style: TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                },
            },
            ..Default::default()
        })
        .with(md);

    commands.push_children(container_entity, &[commands.current_entity().unwrap()]);
}
