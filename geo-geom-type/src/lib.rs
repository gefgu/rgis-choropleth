#![warn(
    clippy::unwrap_used,
    clippy::cast_lossless,
    clippy::unimplemented,
    clippy::indexing_slicing,
    clippy::expect_used
)]

bitflags::bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct GeomType: u16 {
        const POINT             = 0b000000001;
        const LINE              = 0b000000010;
        const LINE_STRING       = 0b000000100;
        const POLYGON           = 0b000001000;
        const MULTI_POINT       = 0b000010000;
        const MULTI_LINE_STRING = 0b000100000;
        const MULTI_POLYGON     = 0b001000000;
        const RECT              = 0b010000000;
        const TRIANGLE          = 0b100000000;
    }
}

impl std::fmt::Display for GeomType {
    fn fmt(&self, mut f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            n if *n == GeomType::POINT => write!(&mut f, "Point"),
            n if *n == GeomType::LINE => write!(&mut f, "Line"),
            n if *n == GeomType::LINE_STRING => write!(&mut f, "LineString"),
            n if *n == GeomType::POLYGON => write!(&mut f, "Polygon"),
            n if *n == GeomType::MULTI_POINT => write!(&mut f, "MultiPoint"),
            n if *n == GeomType::MULTI_LINE_STRING => write!(&mut f, "MultiLineString"),
            n if *n == GeomType::MULTI_POLYGON => write!(&mut f, "MultiPolygon"),
            n if *n == GeomType::RECT => write!(&mut f, "Rectangle"),
            n if *n == GeomType::TRIANGLE => write!(&mut f, "Triangle"),
            _ => unreachable!("Unknown geometry type"),
        }
    }
}

pub fn determine<'a>(geometries: impl IntoIterator<Item = &'a geo::Geometry>) -> GeomType {
    geometries.into_iter().fold(GeomType::empty(), |acc, next| {
        acc | match next {
            geo::Geometry::Point(_) => GeomType::POINT,
            geo::Geometry::Line(_) => GeomType::LINE,
            geo::Geometry::LineString(_) => GeomType::LINE_STRING,
            geo::Geometry::Polygon(_) => GeomType::POLYGON,
            geo::Geometry::MultiPoint(_) => GeomType::MULTI_POINT,
            geo::Geometry::MultiLineString(_) => GeomType::MULTI_LINE_STRING,
            geo::Geometry::MultiPolygon(_) => GeomType::MULTI_POLYGON,
            geo::Geometry::Rect(_) => GeomType::RECT,
            geo::Geometry::Triangle(_) => GeomType::TRIANGLE,
            geo::Geometry::GeometryCollection(geometry_collection) => {
                determine(geometry_collection)
            }
        }
    })
}
