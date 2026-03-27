use bevy::{
    image::{ImageArrayLayout, ImageLoaderSettings},
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct Room {
    tile_size: u32,
    room_size: UVec2,
    #[dependency]
    tileset: Handle<Image>,
    tile_data: Vec<Option<TileData>>,
}

impl FromWorld for Room {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let tileset = asset_server.load(
            "images/tileset.png"
            
        );

        let tile_data = vec![
            00, 01, 02, 01, 02, 01, 02, 01, 02, 01, 02, 01, 02, 01, 02, 03, //
            04, 05, 05, 06, 05, 05, 06, 05, 05, 06, 05, 05, 06, 05, 05, 07, //
            08, 09, 09, 09, 09, 09, 09, 09, 09, 09, 09, 09, 09, 09, 09, 11, //
            04, 09, 09, 09, 09, 09, 09, 09, 10, 09, 09, 09, 09, 10, 09, 07, //
            08, 09, 10, 09, 09, 09, 09, 09, 09, 09, 09, 09, 09, 09, 09, 11, //
            04, 09, 09, 09, 09, 09, 09, 09, 09, 09, 09, 09, 09, 09, 09, 07, //
            08, 10, 09, 09, 09, 09, 09, 09, 09, 09, 10, 09, 09, 09, 09, 11, //
            04, 09, 09, 09, 09, 09, 10, 09, 09, 09, 09, 09, 09, 09, 09, 07, //
            12, 13, 14, 13, 14, 13, 14, 13, 14, 13, 14, 13, 14, 13, 14, 15, //
        ]
        .iter()
        .map(|tileset_index: &u16| Some(TileData::from_tileset_index(*tileset_index)))
        .collect();

        Self {
            room_size: (16, 9).into(),
            tile_size: 20,
            tileset,
            tile_data,
        }
    }
}

impl Room {
    pub fn bundle(&self) -> impl Bundle {
        (
            TilemapChunk {
                tile_display_size: UVec2::splat(self.tile_size),
                chunk_size: self.room_size,
                tileset: self.tileset.clone(),
                tileset_grid_size: UVec2 { x: 4, y: 4 },
                ..default()
            },
            TilemapChunkTileData(self.tile_data.clone()),
        )
    }
}
