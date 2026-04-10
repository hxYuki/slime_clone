use bevy::{
    ecs::template::template,
    prelude::*,
    scene2::{Scene, bsn},
};

pub fn title_ui() -> impl Scene {
    bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),

            flex_direction: FlexDirection::Column,
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,

            padding: UiRect::all(px(50.0)),
        }

        Children [
            (
                Node {

                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::End,
                    flex_grow: 1.0,
                }
                Children [
                    (
                        Text::new("Dungeon")
                        text_04B_30(130.0)
                    ),
                    (
                        Text::new("Slime")
                        text_04B_30(130.0)
                    )

                ]
            ),
            (
                Text::new("Press Enter to Start")
                text_04B_30(25.0)
            )
        ]
    }
}

pub fn text_04B_30(font_size: f32) -> impl Scene {
    bsn! {
        template(move |ctx| {
            Ok(TextFont {
                font: ctx.resource::<AssetServer>().load("fonts/04B_30.ttf").into(),
                font_size: font_size.into(),
                ..default()
            })
        })
    }
}
