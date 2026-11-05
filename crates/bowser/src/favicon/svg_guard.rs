//! A bounded SVG subset that cannot expand referenced geometry during rendering.

use roxmltree::{Document, ParsingOptions};
use svgtypes::{PathParser, PointsParser, TransformListParser};

const MAX_BYTES: usize = 64 * 1024;
const MAX_NODES: u32 = 512;
const MAX_DEPTH: usize = 32;
const MAX_GEOMETRY_BYTES: usize = 16 * 1024;
const MAX_SEGMENTS: usize = 1024;
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

/// Reject expansion, costly effects, and unbounded geometry before invoking usvg.
pub(super) fn bounded_svg(source: &[u8]) -> Option<()> {
    if source.len() > MAX_BYTES {
        return None;
    }
    let source = std::str::from_utf8(source).ok()?;
    if source.contains("<!DOCTYPE") {
        return None;
    }
    let document = Document::parse_with_options(
        source,
        ParsingOptions {
            nodes_limit: MAX_NODES,
            ..ParsingOptions::default()
        },
    )
    .ok()?;
    if document.root_element().tag_name().name() != "svg" {
        return None;
    }
    let mut geometry_bytes = 0;
    let mut segments = 0;
    for node in document.descendants().filter(|node| node.is_element()) {
        if node.tag_name().namespace() != Some(SVG_NAMESPACE)
            || !supported_element(node.tag_name().name())
            || node.ancestors().filter(|node| node.is_element()).count() > MAX_DEPTH
        {
            return None;
        }
        for attribute in node.attributes() {
            let name = attribute.name();
            let value = attribute.value();
            if matches!(
                name,
                "href" | "style" | "filter" | "mask" | "clip-path" | "stroke-dasharray"
            ) || name.starts_with("marker-")
            {
                return None;
            }
            if matches!(name, "d" | "points" | "transform" | "gradientTransform") {
                geometry_bytes += value.len();
                if geometry_bytes > MAX_GEOMETRY_BYTES {
                    return None;
                }
                match name {
                    "d" => {
                        for segment in PathParser::from(value) {
                            segment.ok()?;
                            segments += 1;
                            if segments > MAX_SEGMENTS {
                                return None;
                            }
                        }
                    }
                    "points" => {
                        segments += PointsParser::from(value).take(MAX_SEGMENTS + 1).count();
                    }
                    _ => {
                        for transform in TransformListParser::from(value) {
                            transform.ok()?;
                            segments += 1;
                            if segments > MAX_SEGMENTS {
                                return None;
                            }
                        }
                    }
                }
                if segments > MAX_SEGMENTS {
                    return None;
                }
            }
        }
    }
    Some(())
}

fn supported_element(name: &str) -> bool {
    matches!(
        name,
        "svg"
            | "g"
            | "defs"
            | "path"
            | "rect"
            | "circle"
            | "ellipse"
            | "line"
            | "polyline"
            | "polygon"
            | "linearGradient"
            | "radialGradient"
            | "stop"
            | "title"
            | "desc"
    )
}

#[cfg(test)]
#[path = "_tests_/svg_guard_tests.rs"]
mod svg_guard_tests;
