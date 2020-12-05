use rgis_layers::Layers;
use std::convert::TryInto;
use std::fs;
use std::io;

pub fn load(
    geojson_file_path: String,
    layers: &mut Layers,
    source_projection: &'static str,
    target_projection: &'static str,
) -> Vec<rgis_layers::LayerId> {
    log::info!("Opening file: {:?}", geojson_file_path);
    let geojson_file = io::BufReader::new(fs::File::open(&geojson_file_path).expect("TODO"));
    log::info!("Parsing file: {:?}", geojson_file_path);
    let geojson: geojson::GeoJson = serde_json::from_reader(geojson_file).unwrap();
    log::info!("Parsed file: {:?}", geojson_file_path);
    let layer_ids = match geojson {
        geojson::GeoJson::Geometry(g) => {
            load_geojson_geometry(g, layers, None, source_projection, target_projection)
        }
        geojson::GeoJson::Feature(f) => {
            load_geojson_feature(f, layers, source_projection, target_projection)
        }
        geojson::GeoJson::FeatureCollection(f) => {
            let mut layer_ids = vec![];
            for feature in f.features {
                layer_ids.extend(load_geojson_feature(
                    feature,
                    layers,
                    source_projection,
                    target_projection,
                ))
            }
            layer_ids
        }
    };
    log::info!("Loaded file: {:?}", geojson_file_path);
    layer_ids
}

fn load_geojson_feature(
    geojson_feature: geojson::Feature,
    layers: &mut Layers,
    source_projection: &'static str,
    target_projection: &'static str,
) -> Vec<rgis_layers::LayerId> {
    if let Some(geometry) = geojson_feature.geometry {
        load_geojson_geometry(
            geometry,
            layers,
            geojson_feature.properties,
            source_projection,
            target_projection,
        )
    } else {
        vec![]
    }
}

fn load_geojson_geometry(
    geojson_geometry: geojson::Geometry,
    layers: &mut Layers,
    metadata: Option<rgis_layers::Metadata>,
    source_projection: &'static str,
    target_projection: &'static str,
) -> Vec<rgis_layers::LayerId> {
    let geojson_value = geojson_geometry.value;

    match geojson_value {
        g @ geojson::Value::LineString(_) => {
            let g = (g.try_into().ok() as Option<geo_types::LineString<f64>>).unwrap();
            vec![layers.add(
                geo_types::Geometry::LineString(g),
                metadata,
                source_projection,
                target_projection,
            )]
        }
        g @ geojson::Value::Polygon(_) => {
            let g = (g.try_into().ok() as Option<geo_types::Polygon<f64>>).unwrap();
            vec![layers.add(
                geo_types::Geometry::Polygon(g),
                metadata,
                source_projection,
                target_projection,
            )]
        }
        g @ geojson::Value::MultiLineString(_) => {
            let g = (g.try_into().ok() as Option<geo_types::MultiLineString<f64>>).unwrap();
            vec![layers.add(
                geo_types::Geometry::MultiLineString(g),
                metadata,
                source_projection,
                target_projection,
            )]
        }
        g @ geojson::Value::MultiPolygon(_) => {
            let g = (g.try_into().ok() as Option<geo_types::MultiPolygon<f64>>).unwrap();
            vec![layers.add(
                geo_types::Geometry::MultiPolygon(g),
                metadata,
                source_projection,
                target_projection,
            )]
        }
        geojson::Value::GeometryCollection(g) => {
            let mut layer_ids = vec![];
            for geojson_geometry in g {
                layer_ids.extend(load_geojson_geometry(
                    geojson_geometry,
                    layers,
                    metadata.clone(),
                    source_projection,
                    target_projection,
                ));
            }
            layer_ids
        }
        _ => vec![],
    }
}
