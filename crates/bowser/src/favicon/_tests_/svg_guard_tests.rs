//! Regression coverage for the supported SVG complexity boundary.

use super::bounded_svg;

#[test]
fn rejects_small_exponentially_expanding_geometry() {
    let mut body = String::from("<defs><g id='a0'><rect width='1' height='1'/></g>");
    for depth in 1..=16 {
        let parent = depth - 1;
        body.push_str(&format!(
            "<g id='a{depth}'><use href='#a{parent}'/><use href='#a{parent}'/></g>"
        ));
    }
    body.push_str("</defs><use href='#a16'/>");
    assert!(bounded_svg(svg(&body).as_bytes()).is_none());
}

#[test]
fn bounds_implicit_path_segments_and_polygon_points() {
    for body in [
        format!("<path d='M0 0 {}'/>", "1 1 ".repeat(1024)),
        format!("<polygon points='{}'/>", "1,1 ".repeat(1025)),
        format!("<path d='M0 0{}'/>", " ".repeat(16 * 1024)),
    ] {
        assert!(bounded_svg(svg(&body).as_bytes()).is_none());
    }
    assert!(bounded_svg(svg("<path d='M0 0L32 32Z'/>").as_bytes()).is_some());
}

#[test]
fn bounds_xml_size_nodes_and_depth() {
    for body in [
        "<g/>".repeat(512),
        format!("{}{}", "<g>".repeat(32), "</g>".repeat(32)),
        format!("<desc>{}</desc>", "a".repeat(64 * 1024)),
    ] {
        assert!(bounded_svg(svg(&body).as_bytes()).is_none());
    }
}

#[test]
fn rejects_effects_embedded_resources_css_and_entities() {
    for body in [
        "<filter/>",
        "<pattern/>",
        "<mask/>",
        "<clipPath/>",
        "<image href='data:image/png;base64,AA=='/>",
        "<text>hello</text>",
        "<style>path { fill: red }</style>",
        "<path style='stroke-dasharray:1 1'/>",
        "<path stroke-dasharray='1 1'/>",
        "<linearGradient href='#other'/>",
    ] {
        assert!(bounded_svg(svg(body).as_bytes()).is_none(), "{body}");
    }
    let dtd = format!(
        "<!DOCTYPE svg [<!ENTITY x 'hello'>]>{}",
        svg("<desc>&x;</desc>")
    );
    assert!(bounded_svg(dtd.as_bytes()).is_none());
    assert!(bounded_svg(b"\x1f\x8bcompressed SVG").is_none());
}

fn svg(body: &str) -> String {
    format!("<svg xmlns='http://www.w3.org/2000/svg' width='32' height='32'>{body}</svg>")
}
