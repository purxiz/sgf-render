use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    generate_styles();
    generate_tests();
}

fn generate_styles() {
    let outfile_path = Path::new(&env::var("OUT_DIR").unwrap()).join("generated_styles.rs");
    let mut outfile = fs::File::create(outfile_path).unwrap();
    write!(
        outfile,
        r#"/// Automatically generated styles.

use std::collections::HashMap;

use super::GobanStyle;

lazy_static::lazy_static! {{
    pub static ref GENERATED_STYLES: HashMap<&'static str, GobanStyle> = {{
        let mut m = HashMap::new();"#
    )
    .unwrap();

    let styles = std::fs::read_dir("./resources/styles").unwrap();
    for result in styles {
        let path = result.unwrap().path().canonicalize().unwrap();
        if path.extension().and_then(OsStr::to_str) == Some("toml") {
            write_style(&mut outfile, &path);
        }
    }

    write!(
        outfile,
        r#"
        m
    }};
}}
"#
    )
    .unwrap();
}

fn write_style(outfile: &mut fs::File, path: &Path) {
    let data = &std::fs::read_to_string(path)
        .unwrap()
        .parse::<toml::Value>()
        .unwrap();
    let style_name = path
        .with_extension("")
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    write!(
        outfile,
        r#"
        m.insert("{style_name}", GobanStyle {{
            line_color: {line_color}.to_string(),
            line_width: {line_width},
            hoshi_radius: {hoshi_radius},
            background_fill: {background_fill}.to_string(),
            label_color: {label_color}.to_string(),
            black_stone_fill: {black_stone_fill},
            white_stone_fill: {white_stone_fill},
            black_stone_stroke: {black_stone_stroke},
            white_stone_stroke: {white_stone_stroke},
            markup_stroke_width: {markup_stroke_width},
            black_stone_markup_color: {black_stone_markup_color}.to_string(),
            white_stone_markup_color: {white_stone_markup_color}.to_string(),
            empty_markup_color: {empty_markup_color}.to_string(),
            black_stone_selected_color: {black_stone_selected_color}.to_string(),
            white_stone_selected_color: {white_stone_selected_color}.to_string(),
            empty_selected_color: {empty_selected_color}.to_string(),
            defs: {defs},
        }});"#,
        style_name = style_name,
        line_color = data["line_color"],
        line_width = data["line_width"],
        hoshi_radius = data["hoshi_radius"],
        background_fill = data["background_fill"],
        label_color = data["label_color"],
        black_stone_fill = display_option(data.get("black_stone_fill")),
        white_stone_fill = display_option(data.get("white_stone_fill")),
        black_stone_stroke = display_option(data.get("black_stone_stroke")),
        white_stone_stroke = display_option(data.get("white_stone_stroke")),
        markup_stroke_width = data["markup_stroke_width"],
        black_stone_markup_color = data["black_stone_markup_color"],
        white_stone_markup_color = data["white_stone_markup_color"],
        empty_markup_color = data["empty_markup_color"],
        black_stone_selected_color = data["black_stone_selected_color"],
        white_stone_selected_color = data["white_stone_selected_color"],
        empty_selected_color = data["empty_selected_color"],
        defs = display_option(data.get("defs")),
    )
    .unwrap();
}

fn display_option(x: Option<&toml::Value>) -> String {
    match x {
        Some(value) => format!("Some({}.to_string())", value),
        None => "None".to_string(),
    }
}

fn generate_tests() {
    let outfile_path = Path::new(&env::var("OUT_DIR").unwrap()).join("generated_tests.rs");
    let mut outfile = fs::File::create(outfile_path).unwrap();
    write_tests_header(&mut outfile);

    let test_data = std::fs::read_dir("./tests/data").unwrap();
    for result in test_data {
        let entry = result.unwrap();
        if entry.metadata().unwrap().is_dir() {
            write_test(&mut outfile, &entry);
        }
    }
}

fn write_tests_header(outfile: &mut fs::File) {
    write!(
        outfile,
        r#"/// Automatically generated tests.

use sgf_render::make_svg;
use sgf_render::args;
"#,
    )
    .unwrap();
}

fn write_test(outfile: &mut fs::File, dir: &fs::DirEntry) {
    let dir = dir.path().canonicalize().unwrap();
    let path = dir.display();
    let separator = std::path::MAIN_SEPARATOR;
    let test_name = dir.file_name().unwrap().to_string_lossy();

    write!(
        outfile,
        r#"
#[test]
fn {test_name}() {{
    let mut arguments = shell_words::split(include_str!(r"{path}{separator}options.txt")).unwrap();
    if let Some(i) = arguments.iter().position(|s| s == "--custom-style") {{
        arguments[i + 1] = format!(r"{path}{separator}{{}}", arguments[i + 1]);
    }}
    let matches = args::build_opts().parse(&arguments).unwrap();
    let options = args::extract_make_svg_options(&matches).unwrap();
    let input = include_str!(r"{path}{separator}input.sgf");
    let expected = include_str!(r"{path}{separator}output.svg");

    let svg = make_svg(input, &options).unwrap();
    let mut buffer: Vec<u8> = vec![];
    svg.write_to(&mut buffer).unwrap();
    let result = std::str::from_utf8(&buffer).unwrap();

    assert_eq!(result, expected);
}}
        "#,
        test_name = test_name,
        path = path,
        separator = separator,
    )
    .unwrap();
}
