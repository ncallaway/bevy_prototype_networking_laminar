use bevy::prelude::*;

use smallvec::SmallVec;

use super::game::Message;

const STAGE_UI_BEFORE: &str = "ui_before";
const STAGE_UI_AFTER: &str = "ui_after";
const UI_BACKGROUND: Color = Color::rgba(0.0, 0.0, 0.0, 0.9);

struct ContainerEntity(Entity);

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

struct MessageDisplay {
    ordinal: u8,
}

struct MessageContainer;

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

pub fn build(app: &mut AppBuilder) {
    app.init_resource::<ButtonMaterials>()
        .init_resource::<Handle<Font>>()
        .add_resource(ContainerEntity(Entity::new()))
        .add_startup_system(setup_ui.system())
        .add_stage_before(stage::UPDATE, STAGE_UI_BEFORE)
        .add_stage_after(stage::UPDATE, STAGE_UI_AFTER)
        .add_system_to_stage(STAGE_UI_BEFORE, button_hover_system.system())
        .add_system(message_display_sync_system.system())
        .add_system_to_stage(STAGE_UI_AFTER, message_display_ordering_system.system());
}

fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut font_handle: ResMut<Handle<Font>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut container: ResMut<ContainerEntity>,
    button_materials: Res<ButtonMaterials>,
) {
    *font_handle = asset_server
        .load("assets/fonts/FiraMono-Medium.ttf")
        .unwrap();

    let e = Entity::new();
    container.0 = e;

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
                            value: "Send a message".to_string(),
                            font: *font_handle,
                            style: TextStyle {
                                font_size: 12.0,
                                color: Color::rgb(0.8, 0.8, 0.8),
                            },
                        },
                        ..Default::default()
                    });
                })
                // messages box
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
                        .spawn_as_entity(
                            e,
                            NodeComponents {
                                style: Style {
                                    size: Size::new(Val::Percent(100.0), Val::Auto),
                                    justify_content: JustifyContent::FlexStart,
                                    flex_direction: FlexDirection::Column,
                                    ..Default::default()
                                },
                                material: materials.add(UI_BACKGROUND.into()),
                                ..Default::default()
                            },
                        )
                        .with(MessageContainer)
                        .spawn(TextComponents {
                            style: Style {
                                align_self: AlignSelf::Center,
                                ..Default::default()
                            },
                            text: Text {
                                value: "MESSAGES ".to_string(),
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

fn message_display_sync_system(
    mut commands: Commands,
    font_handle: Res<Handle<Font>>,
    mut messages: Query<&Message>,
    mut display_containers: Query<(Entity, &MessageContainer)>,
    mut message_displays: Query<(Entity, &mut MessageDisplay, &mut Text)>,
) {
    let mut display_borrow = message_displays.iter();
    let mut display_iter = display_borrow.into_iter();

    for msg in &mut messages.iter() {
        let has_display = display_iter.next();

        match has_display {
            Some((_, mut display, mut text)) => {
                // oh no! A string allocation in a system loop! That's not great. I don't care
                // about the performance of this, but please don't copy this somewhere real.
                let desired_text = format!("[{}] {}", msg.from, msg.message);
                if text.value != desired_text || display.ordinal != msg.ordinal {
                    text.value = format!("[{}] {}", msg.from, msg.message);
                    display.ordinal = msg.ordinal;
                }
            }
            None => spawn_message_display(
                &mut commands,
                *font_handle,
                &mut display_containers,
                format!("[{}] {}", msg.from, msg.message),
                msg.ordinal,
            ),
        }
    }

    // remove the remaining entities
    for (e, _, _) in display_iter {
        commands.despawn(e);
    }
}

fn message_display_ordering_system(
    mut containers: Query<(Entity, &MessageContainer, &mut Children)>,
    mut message_display: Query<(Entity, &mut MessageDisplay)>,
) {
    // get the children of the message container
    let mut container_borrow = containers.iter();
    let mut container_children = match container_borrow.into_iter().next() {
        Some((_, _, c)) => c,
        None => return,
    };

    // (_, _, mut container_children) = ?;

    let mut last = None;
    let mut needs_reorder = false;

    for child in container_children.iter() {
        let d = message_display.get::<MessageDisplay>(*child).unwrap();

        if let Some(prior) = last {
            if d.ordinal > prior {
                needs_reorder = true;
            }
        }

        last = Some(d.ordinal);
    }

    if needs_reorder {
        let mut display_borrow = message_display.iter();
        let mut sorted_displays: Vec<(Entity, Mut<MessageDisplay>)> =
            display_borrow.into_iter().collect();

        sorted_displays.sort_by(|(_, a), (_, b)| b.ordinal.cmp(&a.ordinal));

        let sorted_entities: Vec<Entity> = sorted_displays.iter().map(|(e, _)| *e).collect();
        container_children.0 = SmallVec::from_slice(&sorted_entities[..]);
    }
}

fn spawn_message_display(
    commands: &mut Commands,
    font_handle: Handle<Font>,
    display_containers: &mut Query<(Entity, &MessageContainer)>,
    value: String,
    ordinal: u8,
) {
    // have to have a message container, or we die, sorry.
    let (container_entity, _) = display_containers.iter().into_iter().next().unwrap();
    spawn_message_display_with_entity(commands, &font_handle, container_entity, value, ordinal);
}

fn spawn_message_display_with_entity(
    commands: &mut Commands,
    font_handle: &Handle<Font>,
    container_entity: Entity,
    value: String,
    ordinal: u8,
) {
    let md = MessageDisplay { ordinal };

    let e = Entity::new();
    commands
        .spawn_as_entity(
            e,
            TextComponents {
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
            },
        )
        .with(md)
        .push_children(container_entity, &[e]);
}
