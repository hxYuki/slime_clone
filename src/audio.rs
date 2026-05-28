use bevy::{audio::Volume, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource::<ChannelVolume>(ChannelVolume {
        music_volume: Volume::Linear(1.),
        sound_effect_volume: Volume::Linear(1.),
    })
    .add_systems(
        Update,
        apply_global_volume
            .run_if(resource_changed::<GlobalVolume>.or_eager(resource_changed::<ChannelVolume>)),
    )
    .add_observer(
        |ev: On<Add, AudioSink>,
         global_volume: Res<GlobalVolume>,
         channel_volume: Res<ChannelVolume>,
         mut q_sfx: Query<
            (&PlaybackSettings, &mut AudioSink),
            (With<SoundEffect>, Without<Music>),
        >,
         mut q_music: Query<
            (&PlaybackSettings, &mut AudioSink),
            (With<Music>, Without<SoundEffect>),
        >| {
            let Ok((playback, mut sink)) = q_music
                .get_mut(ev.entity)
                .or_else(|_| q_sfx.get_mut(ev.entity))
            else {
                return;
            };

            sink.set_volume(global_volume.volume * channel_volume.music_volume * playback.volume);
        },
    );
}

#[derive(Resource, Reflect, Default)]
pub struct ChannelVolume {
    pub music_volume: Volume,
    pub sound_effect_volume: Volume,
}

/// An organizational marker component that should be added to a spawned [`AudioPlayer`] if it's in the
/// general "music" category (e.g. global background music, soundtrack).
///
/// This can then be used to query for and operate on sounds in that category.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct Music;

/// A music audio instance.
pub fn music_bundle(handle: Handle<AudioSource>) -> impl Bundle {
    (AudioPlayer(handle), PlaybackSettings::LOOP, Music)
}

pub fn music(path: &'static str) -> impl Scene {
    bsn! {
        AudioPlayer({ path })
        PlaybackSettings::LOOP
        Music
    }
}

/// An organizational marker component that should be added to a spawned [`AudioPlayer`] if it's in the
/// general "sound effect" category (e.g. footsteps, the sound of a magic spell, a door opening).
///
/// This can then be used to query for and operate on sounds in that category.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct SoundEffect;

/// A sound effect audio instance.
pub fn sound_effect_bundle(handle: Handle<AudioSource>) -> impl Bundle {
    (AudioPlayer(handle), PlaybackSettings::DESPAWN, SoundEffect)
}

pub fn sound_effect(path: &'static str) -> impl Scene {
    bsn! {
        AudioPlayer({ path })
        PlaybackSettings::DESPAWN
        SoundEffect
    }
}

/// [`GlobalVolume`] doesn't apply to already-running audio entities, so this system will update them.
fn apply_global_volume(
    global_volume: Res<GlobalVolume>,
    channel_volume: Res<ChannelVolume>,
    mut q_sfx: Query<(&PlaybackSettings, &mut AudioSink), (With<SoundEffect>, Without<Music>)>,
    mut q_music: Query<(&PlaybackSettings, &mut AudioSink), (With<Music>, Without<SoundEffect>)>,
) {
    for (playback, mut sink) in &mut q_music {
        sink.set_volume(global_volume.volume * channel_volume.music_volume * playback.volume);
    }
    for (playback, mut sink) in &mut q_sfx {
        sink.set_volume(
            global_volume.volume * channel_volume.sound_effect_volume * playback.volume,
        );
    }
}
