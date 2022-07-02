use bevy::prelude::*;

fn handle_toggle_layer_visibility_events(
    mut toggle_layer_visibility_event_reader: bevy::ecs::event::EventReader<
        rgis_events::ToggleLayerVisibilityEvent,
    >,
    mut layer_became_visible_event_writer: bevy::ecs::event::EventWriter<
        rgis_events::LayerBecameVisibleEvent,
    >,
    mut layer_became_hidden_event_writer: bevy::ecs::event::EventWriter<
        rgis_events::LayerBecameHiddenEvent,
    >,
    mut layers: ResMut<crate::Layers>,
) {
    for event in toggle_layer_visibility_event_reader.iter() {
        let layer = match layers.get_mut(event.0) {
            Some(l) => l,
            None => {
                bevy::log::warn!("Could not find layer");
                continue;
            }
        };
        layer.visible = !layer.visible;
        if layer.visible {
            layer_became_visible_event_writer.send(rgis_events::LayerBecameVisibleEvent(event.0));
        } else {
            layer_became_hidden_event_writer.send(rgis_events::LayerBecameHiddenEvent(event.0));
        }
    }
}

fn handle_update_color_events(
    mut update_events: bevy::ecs::event::EventReader<rgis_events::UpdateLayerColorEvent>,
    mut updated_events: bevy::ecs::event::EventWriter<rgis_events::LayerColorUpdatedEvent>,
    mut layers: ResMut<crate::Layers>,
) {
    for event in update_events.iter() {
        let layer = match layers.get_mut(event.0) {
            Some(l) => l,
            None => {
                bevy::log::warn!("Could not find layer");
                continue;
            }
        };
        layer.color = event.1;
        updated_events.send(rgis_events::LayerColorUpdatedEvent(event.0));
    }
}

fn handle_delete_layer_events(
    mut delete_layer_event_reader: bevy::ecs::event::EventReader<rgis_events::DeleteLayerEvent>,
    mut layer_deleted_event_writer: bevy::ecs::event::EventWriter<rgis_events::LayerDeletedEvent>,
    mut layers: ResMut<crate::Layers>,
) {
    for event in delete_layer_event_reader.iter() {
        layers.remove(event.0);
        layer_deleted_event_writer.send(rgis_events::LayerDeletedEvent(event.0));
    }
}

fn handle_move_layer_events(
    mut move_layer_event_reader: bevy::ecs::event::EventReader<rgis_events::MoveLayerEvent>,
    mut layer_z_index_updated_event_writer: bevy::ecs::event::EventWriter<
        rgis_events::LayerZIndexUpdatedEvent,
    >,
    mut layers: ResMut<crate::Layers>,
) {
    for event in move_layer_event_reader.iter() {
        let (_, old_z_index) = match layers.get_with_z_index(event.0) {
            Some(result) => result,
            None => {
                bevy::log::warn!("Could not find layer");
                continue;
            }
        };

        let new_z_index = match event.1 {
            rgis_events::MoveDirection::Up => old_z_index + 1,
            rgis_events::MoveDirection::Down => old_z_index - 1,
        };

        let other_layer_id = match layers.data.get(new_z_index) {
            Some(layer) => layer.id,
            None => {
                bevy::log::warn!("Could not find layer");
                continue;
            }
        };

        layers.data.swap(old_z_index, new_z_index);

        layer_z_index_updated_event_writer.send(rgis_events::LayerZIndexUpdatedEvent(event.0));
        layer_z_index_updated_event_writer
            .send(rgis_events::LayerZIndexUpdatedEvent(other_layer_id));
    }
}

fn handle_map_clicked_events(
    mut map_clicked_event_reader: bevy::ecs::event::EventReader<rgis_events::MapClickedEvent>,
    mut render_message_event_writer: bevy::ecs::event::EventWriter<rgis_events::RenderMessageEvent>,
    layers: Res<crate::Layers>,
) {
    for event in map_clicked_event_reader.iter() {
        if let Some(feature) = layers.feature_from_click(event.0) {
            render_message_event_writer.send(rgis_events::RenderMessageEvent(format!(
                "{:?}",
                feature.properties
            )));
        }
    }
}

fn handle_create_layer_events(
    mut create_layer_events: ResMut<bevy::ecs::event::Events<rgis_events::CreateLayerEvent>>,
    mut layer_created_event_writer: EventWriter<rgis_events::LayerCreatedEvent>,
    mut layers: ResMut<crate::Layers>,
) {
    for event in create_layer_events.drain() {
        match layers.add(event.unprojected_geometry, event.name, event.source_crs) {
            Ok(layer_id) => {
                layer_created_event_writer.send(rgis_events::LayerCreatedEvent(layer_id))
            }
            Err(e) => bevy::log::error!("Encountered error when creating layer: {:?}", e),
        }
    }
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(handle_toggle_layer_visibility_events)
        .with_system(handle_update_color_events)
        .with_system(handle_move_layer_events)
        .with_system(handle_delete_layer_events)
        .with_system(handle_map_clicked_events)
        .with_system(handle_create_layer_events)
}
