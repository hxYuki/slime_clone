use bevy::{
    asset::HandleTemplate,
    ecs::template::template,
    image::TextureAtlasTemplate,
    prelude::*,
    scene::{Scene, bsn},
    text::FontSourceTemplate,
    ui::widget::ImageNodeTemplate,
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
                TextShadow
                Children [
                    (
                        Text::new("Dungeon")
                        font_04B_30(130.0)
                    ),
                    (
                        Text::new("Slime")
                        font_04B_30(130.0)
                    )

                ]
            ),
                (Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceAround,
                    width: percent(100.0),
                }
                Children [

                    (
                        styled_button(bsn_list! {
                            Text::new("Start")
                            button_text_style()

                        })
                    ),
                    (
                        styled_button(bsn_list! {
                            Text::new("Option")
                            button_text_style()
                        })
                    )
                ])
        ]
    }
}
pub fn button_text_style() -> impl Scene {
    bsn! {
    TextColor(Color::srgba_u8(7,14,120,255))
    TextLayout::new_with_justify(Justify::Center)
    Node {
        margin: UiRect::axes(px(30.0), px(2.0)),
    }
    font_04B_30(25.0)
    }
}
pub fn styled_button(children: impl SceneList) -> impl Scene {
    let slicer = TextureSlicer {
        border: BorderRect::axes(10.0, 6.0),
        max_corner_scale: 4.0,
        ..default()
    };

    let texture_atlas = TextureAtlasLayout::from_grid(UVec2::new(65, 13), 1, 3, None, None);

    bsn! {
        Button
        image_node_with_texture_atlas("images/button.png", texture_atlas, NodeImageMode::Sliced(slicer))

        Node{
            // padding: UiRect::axes(px(10.0), px(5.0)),
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            justify_items: JustifyItems::Center,
        }
        Children [
            {children}
        ]
    }
}

// #[derive(Component, FromTemplate)]
// struct TestComponent {
//     texture_atlas: TextureAtlas,
//     #[template(built_in)]
//     texture_atlas_option: Option<TextureAtlas>,
//
//     #[template(built_in)]
//     image: Option<Handle<Image>>,
// }
//
// pub fn button() -> impl Scene {
//     bsn! {
//         template(|ctx|{
//             Ok(ImageNode {
//                 image: ctx.resource::<AssetServer>().load("images/button.png"),
//                 image_mode: NodeImageMode::Sliced(TextureSlicer {
//                     border: BorderRect::axes(10.0, 3.0),
//                     ..default()
//                 }),
//                 texture_atlas: Some(TextureAtlas {
//                     layout: ctx.resource_mut::<Assets<TextureAtlasLayout>>().add(
//                         TextureAtlasLayout::from_grid(UVec2::new(65, 12), 1, 3, None, None)
//                     ),
//                     index: 0,
//                 }),
//                 ..default()
//             })
//         })
//     }
// }

pub fn font_04B_30(font_size: f32) -> impl Scene {
    bsn! {
        TextFont {
            font: FontSourceTemplate::Handle("fonts/04B_30.ttf"),
            font_size: {FontSize::Px(font_size)}
        }
    }
}

pub fn image_node_with_texture_atlas(
    path: &'static str,
    layout: TextureAtlasLayout,
    image_mode: NodeImageMode,
) -> impl Scene {
    bsn! {
        template(move |ctx|{
            Ok(ImageNode {
                image: ctx.resource::<AssetServer>().load(path),
                texture_atlas: Some(TextureAtlas {
                    layout: ctx.resource_mut::<Assets<TextureAtlasLayout>>().add(layout.clone()),
                    index: 0,
                }),
                image_mode: image_mode.clone(),
                ..default()
            })
        })
    }
}
