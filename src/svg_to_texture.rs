// modified version of https://github.com/macnelly/quad-svg
// can't just install the package cause it installs an old version of macroquad and breaks shit
use std::fs;
use macroquad::{prelude::ImageFormat, texture::Texture2D};
use resvg::usvg_text_layout::{fontdb, TreeTextToPath};

const SIZE: u32 = 200;

/*
    rasterize svg to png image
*/
pub fn svg_to_png(svg_str: &str) -> Vec<u8> {
    let opt = resvg::usvg::Options::default();
    let mut tree = resvg::usvg::Tree::from_str(svg_str, &opt).unwrap();
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();
    tree.convert_text(&fontdb, opt.keep_named_groups);
    let pixmap_size = tree.size.to_screen_size();
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(SIZE, SIZE).unwrap();

    resvg::render(
        &tree,
        resvg::usvg::FitTo::Width(SIZE),
        resvg::tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
        .unwrap();
    pixmap.encode_png().unwrap()
}

/*
    rasterize svg and create Texture2D
*/
pub fn svg_to_texture(svg_str: &str) -> Texture2D {
    let png_data = svg_to_png(&svg_str);
    Texture2D::from_file_with_format(&png_data, Some(ImageFormat::Png))
}

pub fn texture_from_file(file: &str) -> Texture2D {
    svg_to_texture(fs::read_to_string(file).unwrap().as_str())
}