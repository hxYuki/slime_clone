mod options;
mod title;

use std::time::Duration;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        lifecycle::HookContext,
        observer::On,
        relationship::RelationshipSourceCollection,
        system::{SystemHandle, SystemHandleTemplate, system_value},
        template::FromTemplate,
        world::DeferredWorld,
    },
    image::TextureAtlasTemplate,
    picking::hover::Hovered,
    prelude::*,
    scene::{Scene, bsn, template_value},
    state::{state::States, state_scoped::DespawnOnExit},
    ui::{Node, percent},
    ui_widgets::{Slider, SliderRange, SliderThumb, SliderValue, TrackClick},
};
pub use options::*;
pub use title::*;

use crate::{
    region_color_screen_material::UiGreyout,
    resource_notify::ResourceChangeHandler,
    sprite_animation::{FrameAnimation, FrameAnimationSet, PlaybackMode},
};

pub fn ui_root() -> impl Scene {
    bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
        }
    }
}

pub fn ui_root_despawn_on_exit<S: States + Default + Clone + Unpin>(state: S) -> impl Scene {
    bsn! {
        ui_root()
        template_value(DespawnOnExit(state))
    }
}

pub fn plugin(app: &mut App) {
    // app.init_resource::<RegisterQuery>();

    app
        // .add_observer(handle_button_group_focus)
        // .add_systems(
        //     Update,
        //     (switch_button_focus_animation, trigger_focused_button),
        // )
        .add_systems(
            Update,
            (update_slider_thumb_position, trigger_focused_button),
        );
}

#[derive(Component, Debug, FromTemplate)]
#[relationship(relationship_target = FocusGroup)]
#[require(GroupFocused)]
pub struct InFocusGroup(pub Entity);

#[derive(Component, Debug, FromTemplate)]
#[relationship_target(relationship = InFocusGroup)]
#[component(on_add = focus_group_observer)]
pub struct FocusGroup(Vec<Entity>);

fn focus_group_observer(mut world: DeferredWorld, ctx: HookContext) {
    world
        .commands()
        .entity(ctx.entity)
        .observe(group_focus_change);
}

#[derive(Component, Debug, Clone, Default)]
pub struct GroupFocused(pub bool);

#[derive(Component, FromTemplate)]
pub struct Callback(SystemHandle);

pub fn press_trigger_callback(
    ev: On<Pointer<Press>>,
    mut command: Commands,
    buttons: Query<&Callback>,
) {
    buttons
        .get(ev.entity)
        .map(|action| {
            command.run_system(&action.0);
        })
        .expect(
            "You should only use on_press_trigger_callback on entities with a Callback component",
        );
}

#[derive(EntityEvent, Debug)]
struct GroupFocusChangedEvent {
    entity: Entity,
    focus: Entity,
}

#[derive(EntityEvent, Debug)]
struct RespondFocusChange {
    entity: Entity,
    focused: bool,
}

fn pointer_focus_control(
    ev: On<Pointer<Enter>>,
    focusables: Query<(Entity, &InFocusGroup)>,
    mut commands: Commands,
) {
    if let Ok((_, in_group)) = focusables.get(ev.entity) {
        commands
            .entity(in_group.0)
            .trigger(|e| GroupFocusChangedEvent {
                entity: e,
                focus: ev.entity,
            });
    }
}
fn group_focus_change(
    ev: On<GroupFocusChangedEvent>,
    groups: Query<&FocusGroup>,
    mut focusables: Query<(&mut GroupFocused, &InFocusGroup)>,
    mut commands: Commands,
) {
    let Ok(group) = groups.get(ev.entity) else {
        return;
    };

    group.0.iter().for_each(|b| {
        if let Ok((mut focused, _)) = focusables.get_mut(b) {
            focused.0 = b == ev.focus;
        }

        commands.entity(b).trigger(|e| RespondFocusChange {
            entity: e,
            focused: b == ev.focus,
        });
    });
}

fn switch_button_focus_animation(
    ev: On<RespondFocusChange>,
    mut focusables: Query<&mut FrameAnimationSet<ButtonAnimation>>,
) {
    if let Ok(mut animation) = focusables.get_mut(ev.entity) {
        if ev.focused {
            animation.set_state(ButtonAnimation::Focused);
        } else {
            animation.set_state(ButtonAnimation::Default);
        }
    }
}

fn trigger_focused_button(
    input: Res<ButtonInput<KeyCode>>,
    gamepads: Query<(Entity, &Gamepad)>,

    button_groups: Query<(Entity, &FocusGroup, &InheritedVisibility)>,
    buttons: Query<(Entity, &InFocusGroup, &GroupFocused, &Callback), With<Button>>,

    mut command: Commands,
) {
    let mut trigger = false;
    gamepads.iter().take(1).for_each(|(entity, gamepad)| {
        if gamepad.just_pressed(GamepadButton::South) {
            trigger = true;
        }
    });

    if input.just_pressed(KeyCode::Enter) {
        trigger = true;
    }

    if !trigger {
        return;
    }

    let active_groups = button_groups
        .iter()
        .filter(|(_, _, visibility)| visibility.get());
    match active_groups.clone().count() {
        0 => {}
        1 => {
            let (_, group, _) = active_groups.into_iter().next().unwrap();
            let focused_button = group
                .0
                .iter()
                .filter_map(|b| buttons.get(b).ok())
                .find(|(_, _, GroupFocused(focused), _)| *focused);

            if let Some((_, _, _, system_callback)) = focused_button {
                command.run_system(&system_callback.0);
            }
        }
        _ => {
            warn!("Multiple active button groups, can't determine which one to trigger");
        }
    }
}

/// A slider component should listen a data source as SliderValue, and set it on ValueChange.

#[derive(SceneComponent, Default, Clone)]
#[scene(StyledSliderProps)]
pub struct StyledSlider;

pub struct StyledSliderProps {
    caption: String,
    slider_modifiers: Box<dyn Scene>,
    resource_change_handler: SystemHandleTemplate<In<Entity>>,
}
impl Default for StyledSliderProps {
    fn default() -> Self {
        Self {
            caption: "Slider".to_string(),
            slider_modifiers: Box::new(bsn! {

                                       }),
            resource_change_handler: system_value(|_entity: In<Entity>| {}),
        }
    }
}

impl StyledSlider {
    pub fn scene(props: StyledSliderProps) -> impl Scene {
        let slicer = TextureSlicer {
            border: BorderRect::axes(10.0, 10.0),
            max_corner_scale: 4.0,
            ..default()
        };
        let texture_atlas =
            TextureAtlasLayout::from_grid(UVec2::new(15, 15), 1, 3, Some(UVec2::new(0, 1)), None);

        bsn! {
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::SpaceBetween,
                row_gap: px(20),
                padding: UiRect::axes(px(40), px(30)),
            }
            ImageNode {
                image: "images/panel-background.png",
                image_mode: NodeImageMode::Sliced(slicer),
                texture_atlas: { TextureAtlasTemplate {
                    layout: asset_value(texture_atlas),
                    index: 1,
                } },
            }
            on(pointer_focus_control)
            UiGreyout {
                enabled: true,
            }
            on(|focus: On<RespondFocusChange>, mut commands: Commands| {
                commands
                    .entity(focus.entity)
                    .insert(UiGreyout {
                        enabled: !focus.focused,
                    });
            })
            Children [
                (
                    Node {
                        align_self: AlignSelf::FlexStart,
                    }
                    Text::new(props.caption)
                    font_04B_30(px(80.0))
                ),
                (
                    Node {
                        width: percent(100),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        justify_items: JustifyItems::Stretch,
                    }
                    Children [
                        slider_knot(KnotPosition::Left, "OFF".to_string(), px(35.0)),
                        (
                            slider_ui()
                            ~Node {
                                flex_grow: 1.0,
                            }
                            { props.slider_modifiers }
                            ResourceChangeHandler({ props.resource_change_handler })
                        ),
                        slider_knot(KnotPosition::Right, "MAX".to_string(), px(35.0)),
                    ]
                ),
            ]
        }
    }
}

enum KnotPosition {
    Left,
    Right,
}
fn slider_knot(position: KnotPosition, text: String, font_size: Val) -> impl Scene {
    let altas_index = match position {
        KnotPosition::Left => 2,
        KnotPosition::Right => 3,
    };
    let text_margin = match position {
        KnotPosition::Left => UiRect::left(px(10.0)),
        KnotPosition::Right => UiRect::right(px(5.0)).with_left(px(10.0)),
    };
    bsn! {
        ~Node {
            padding: UiRect::all(px(5.0)),
            display: Display::Flex,
            justify_content: JustifyContent::Center,
        }
        ImageNode {
            image: "images/slider.png",
            image_mode: NodeImageMode::Sliced({ TextureSlicer {
                border: BorderRect::axes(6.0, 6.0),
                max_corner_scale: 4.0,
                ..default()
            } }),
            texture_atlas: { TextureAtlasTemplate {
                layout: asset_value(
                    TextureAtlasLayout::from_grid(
                        UVec2::new(12, 10),
                        2,
                        2,
                        Some(UVec2::new(1, 1)),
                        None,
                    ),
                ),
                index: altas_index,
            } },
        }
        Children [(
            ~Node {
                margin: { text_margin },
            }
            Text::new(text)
            TextColor(Color::srgb_u32(0x465682))
            font_04B_30(font_size)
        )]
    }
}

pub fn slider_ui() -> impl Scene {
    bsn! {
        #Slider
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            justify_items: JustifyItems::Stretch,
            column_gap: px(4),
            width: percent(100),
        }
        Hovered
        Slider {
            track_click: TrackClick::Snap,
        }
        SliderValue(100.0)
        SliderRange::new(0.0, 100.0)
        Children [(
            Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(-4),
                top: px(0),
                bottom: px(0),
                margin: UiRect::axes(px(0), px(0)),
                padding: UiRect::axes(px(0), px(0)),
                border: UiRect::axes(px(0), px(4)),
            }
            BorderColor::from(Color::srgb_u32(0x465682))
            Children [
                (
                    Node {
                        display: Display::Flex,
                        width: px(12),
                        height: percent(100),
                        position_type: PositionType::Absolute,
                        left: percent(0),
                        border_radius: BorderRadius::MAX,
                    }
                    SliderThumb
                ),
                (
                    ThumbOfSlider(#Slider)
                    Node {
                        display: Display::Flex,
                        width: percent(100),
                        height: percent(100),
                        position_type: PositionType::Absolute,
                        left: px(0),
                        right: px(0),
                    }
                    BackgroundColor(Color::WHITE)
                ),
            ]
        )]
    }
}

#[derive(Component, FromTemplate)]
#[relationship(relationship_target = SliderEntityThumb)]
struct ThumbOfSlider(Entity);
#[derive(Component, FromTemplate)]
#[relationship_target(relationship = ThumbOfSlider)]
struct SliderEntityThumb(Entity);

fn update_slider_thumb_position(
    sliders: Query<(&SliderValue, &SliderRange, &SliderEntityThumb), Changed<SliderValue>>,
    mut q_thumb: Query<&mut Node, With<ThumbOfSlider>>,
) {
    sliders
        .iter()
        .for_each(|(value, range, SliderEntityThumb(thumb))| {
            if let Ok(mut node) = q_thumb.get_mut(*thumb) {
                node.width = percent(range.thumb_position(value.0) * 100.0);
            }
        });
}

#[derive(SceneComponent, Default, Clone)]
#[scene(StyledButtonProps)]
pub struct StyledButton;

crate::animation_state! {
    pub enum ButtonAnimation {
        Default,
        Focused,
    }
}

crate::register_frame_animation_state!(ButtonAnimation, image_node);

impl StyledButton {
    pub fn scene(props: StyledButtonProps) -> impl Scene {
        let slicer = TextureSlicer {
            border: BorderRect::axes(10.0, 6.0),
            max_corner_scale: 4.0,
            ..default()
        };

        let texture_atlas =
            TextureAtlasLayout::from_grid(UVec2::new(65, 15), 1, 3, Some(UVec2::new(0, 1)), None);

        let animations = [
            FrameAnimation::new(
                "default".into(),
                0..1,
                Duration::from_millis(300),
                PlaybackMode::Loop,
            ),
            FrameAnimation::new(
                "focused".into(),
                1..3,
                Duration::from_millis(300),
                PlaybackMode::Loop,
            ),
        ];

        bsn! {
            Button
            ImageNode {
                image: "images/button.png",
                image_mode: NodeImageMode::Sliced(slicer),
                texture_atlas: { TextureAtlasTemplate {
                    layout: asset_value(texture_atlas),
                    index: 0,
                } },
            }
            template_value(FrameAnimationSet::<
                ButtonAnimation,
            >::new(ButtonAnimation::Default, animations.clone()))
            on(pointer_focus_control)
            on(switch_button_focus_animation)
            on(press_trigger_callback)
            Callback({ props.callback })
            Node {
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                justify_items: JustifyItems::Center,
            }
            Children [
                { props.caption },
            ]
        }
    }
}
pub struct StyledButtonProps {
    pub caption: Box<dyn SceneList>,
    pub callback: SystemHandleTemplate,
}
impl Default for StyledButtonProps {
    fn default() -> Self {
        Self {
            caption: Box::new(bsn! {
                                  Text::new("Button")
                              }),
            callback: system_value(|| {}),
        }
    }
}
