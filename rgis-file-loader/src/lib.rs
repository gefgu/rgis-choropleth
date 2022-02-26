use bevy::app::Events;
use bevy::prelude::*;

mod geojson;

struct SpawnedLayers(Vec<rgis_layer_id::LayerId>);

type LoadedGeoJsonFileSender = async_channel::Sender<SpawnedLayers>;
type LoadedGeoJsonFileReceiver = async_channel::Receiver<SpawnedLayers>;

// System
fn load_geojson_file_handler(
    layers: rgis_layers::ResLayers,
    mut load_event_reader: ResMut<Events<rgis_events::LoadGeoJsonFileEvent>>,
    thread_pool: Res<bevy::tasks::AsyncComputeTaskPool>,
    sender: Res<LoadedGeoJsonFileSender>,
) {
    for event in load_event_reader.drain()
    {
        match event {
            rgis_events::LoadGeoJsonFileEvent::FromPath {
                path: geojson_file_path,
                source_srs,
                target_srs,
            } => {
                let layers = layers.clone();
                let sender: LoadedGeoJsonFileSender = sender.clone();
                thread_pool
                    .spawn(async move {
                        let spawned_layers = SpawnedLayers(geojson::load_from_path(
                            geojson_file_path,
                            &mut layers.write().unwrap(),
                            &source_srs,
                            &target_srs,
                        ));
                        sender.send(spawned_layers).await.unwrap();
                    })
                    .detach();
            }
            rgis_events::LoadGeoJsonFileEvent::FromBytes {
                file_name,
                bytes,
                source_srs,
                target_srs,
            } => {
                let sender: LoadedGeoJsonFileSender = sender.clone();
                let layers = layers.clone();
                thread_pool
                    .spawn(async move {
                        let spawned_layers = SpawnedLayers(geojson::load_from_reader(
                            std::io::Cursor::new(bytes),
                            file_name,
                            &mut layers.write().unwrap(),
                            &source_srs,
                            &target_srs,
                        ));
                        sender.send(spawned_layers).await.unwrap();
                    })
                    .detach();
            }
        }
    }
}

fn handle_loaded_layers(
    mut loaded_events: ResMut<Events<rgis_events::LayerLoadedEvent>>,
    receiver: Res<LoadedGeoJsonFileReceiver>,
) {
    while let Ok(layer_ids) = receiver.try_recv() {
        for layer_id in layer_ids.0 {
            loaded_events.send(rgis_events::LayerLoadedEvent(layer_id));
        }
    }
}

fn load_layers_from_cli(
    cli_values: Res<rgis_cli::Values>,
    mut events: ResMut<Events<rgis_events::LoadGeoJsonFileEvent>>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        events.send(rgis_events::LoadGeoJsonFileEvent::FromPath {
            path: "foo".into(),
            source_srs: "EPSG:4326".into(),
            target_srs: "EPSG:3857".into(),
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        for geojson_file_path in &cli_values.geojson_files {
            debug!(
                "sending LoadGeoJsonFile event: {}",
                &geojson_file_path.display(),
            );
            events.send(rgis_events::LoadGeoJsonFileEvent::FromPath {
                path: geojson_file_path.clone(),
                source_srs: cli_values.source_srs.clone(),
                target_srs: cli_values.target_srs.clone(),
            });
        }
    }
}

pub struct RgisFileLoaderPlugin;

impl Plugin for RgisFileLoaderPlugin {
    fn build(&self, app: &mut App) {
        let (sender, receiver): (LoadedGeoJsonFileSender, LoadedGeoJsonFileReceiver) =
            async_channel::unbounded();
        app.insert_resource(sender)
            .insert_resource(receiver)
            .add_startup_system(load_layers_from_cli.system())
            .add_system(load_geojson_file_handler.system())
            .add_system(handle_loaded_layers.system());
    }
}
