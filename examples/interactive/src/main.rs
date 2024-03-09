mod ui;

use bevy::diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
};
use bevy::prelude::*;

use crate::ui::UiPlugin;
use bevy_kira_components::commands::{PauseAudio, PlayAudio};
use bevy_kira_components::kira::track::effect::panning_control::{
    PanningControlBuilder, PanningControlHandle,
};
use bevy_kira_components::kira::track::TrackBuilder;
use bevy_kira_components::kira::tween::Tween;
use bevy_kira_components::tracks::{EffectHandle, Track};
use bevy_kira_components::{Audio, AudioLoaderSettings, AudioPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            AudioPlugin,
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            EntityCountDiagnosticsPlugin,
            UiPlugin,
        ))
        .add_systems(Startup, (init, init_ui))
        .add_systems(Update, (handle_interactive_sound, update_track_panning))
        .run();
}

#[derive(Component)]
struct InteractiveSound;

#[derive(Component)]
struct PanTrack;

fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle { ..default() });
    let audio_file = asset_server
        .load_with_settings("loop.ogg", |settings: &mut AudioLoaderSettings| {
            settings.looping = true
        });
    let mut track = TrackBuilder::new();
    let panning = track.add_effect(PanningControlBuilder::default());
    let track_entity = commands
        .spawn((Track::new(track), EffectHandle(panning), PanTrack))
        .id();
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(25.0)),
            sprite: Sprite {
                color: Color::GRAY,
                ..default()
            },
            ..default()
        },
        Audio::new(audio_file)
            .start_paused(true)
            .in_track(track_entity),
        InteractiveSound,
    ));
}

fn init_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle { ..default() })
        .with_children(|children| {
            children.spawn(TextBundle {
                text: Text::from_section(
                    "Hold Space to play sound",
                    TextStyle {
                        font_size: 16.0,
                        ..default()
                    },
                ),
                style: Style {
                    margin: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                ..default()
            });
        });
}

fn update_track_panning(
    time: Res<Time>,
    mut q: Query<&mut EffectHandle<PanningControlHandle>, With<PanTrack>>,
) {
    let pan = time.elapsed_seconds_f64().sin();
    for mut effect in &mut q {
        if let Err(err) = effect.set_panning(pan, Tween::default()) {
            error!("Cannot update track panning: {err}");
        }
    }
}

fn handle_interactive_sound(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut q: Query<(Entity, &mut Sprite), (With<Audio>, With<InteractiveSound>)>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        for (entity, mut sprite) in &mut q {
            commands.entity(entity).add(PlayAudio(Tween::default()));
            sprite.color = Color::GREEN;
        }
    } else if keyboard.just_released(KeyCode::Space) {
        for (entity, mut sprite) in &mut q {
            commands.entity(entity).add(PauseAudio(Tween::default()));
            sprite.color = Color::GRAY;
        }
    }
}
