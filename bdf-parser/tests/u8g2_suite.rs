extern crate bdf_parser;
extern crate chardet;
extern crate encoding;
extern crate nom;

use chardet::{detect, charset2encoding};
use encoding::DecoderTrap;
use encoding::label::encoding_from_whatwg_label;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self, DirEntry};
use std::io;
use std::io::prelude::*;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use bdf_parser::*;
use nom::*;

// // one possible implementation of walking a directory only visiting files
// fn visit_dirs(dir: &Path, cb: &Fn(&DirEntry)) -> io::Result<()> {
//     if dir.is_dir() {
//         for entry in fs::read_dir(dir)? {
//             let entry = entry?;
//             let path = entry.path();
//             if path.is_dir() {
//                 visit_dirs(&path, cb)?;
//             } else {
//                 cb(&entry);
//             }
//         }
//     }
//     Ok(())
// }

fn collect_font_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                files.push(path.to_path_buf());
            }
        }
    }

    files.sort();

    Ok(files)
}

fn read(path: &Path) -> String {
    // let mut file = File::open(path).expect("Unable to open file");
    // let mut contents = String::new();
    // file.read_to_string(&mut contents)
    //     .expect(&format!("Unable to read file {:?}", path));

    // open text file
    let mut fh = OpenOptions::new()
        .read(true)
        .open(path)
        .expect("Could not open file");
    let mut reader: Vec<u8> = Vec::new();

    // read file
    fh.read_to_end(&mut reader).expect("Could not read file");

    // detect charset of the file
    let result = detect(&reader);
    // result.0 Encode
    // result.1 Confidence
    // result.2 Language

    // decode file into utf-8
    let coder = encoding_from_whatwg_label(charset2encoding(&result.0));
    // if coder.is_some() {
    let utf8reader = coder
        .unwrap()
        .decode(&reader, DecoderTrap::Ignore)
        .expect("Error");
    // }

    // contents

    utf8reader
}

fn test_font_parse(filepath: &Path) -> Result<(), String> {
    let bdf = read(filepath);

    let parser = BDFParser::new(&bdf);

    let out = parser.parse();

    match out {
        IResult::Done(rest, parsed) => {
            // println!("Rest: {:?}", rest);

            if rest.len() > 0 {
                Err(format!("{} remaining bytes to parse", rest.len()))
            } else {
                Ok(())
            }
        }
        IResult::Error(e) => Err(format!("Error")),
        _ => Err(format!("Other error")),
    }
}

#[test]
fn it_parses_all_u8g2_fonts() {
    let fontdir = Path::new("./tests/u8g2/tools/font/bdf")
        .canonicalize()
        .unwrap();

    // println!("Font dir {:?}", fontdir);

    let fonts = collect_font_files(&fontdir).expect("Could not get list of u8g2 fonts");

    // println!("{:?}", fonts);

    let results = fonts.iter().map(|fpath| test_font_parse(fpath));

    let mut num_errors = 0;

    for (font, result) in fonts.iter().zip(results) {
        if result.is_err() {
            num_errors += 1;
        }

        println!(
            "{0: <30} {1:?}",
            font.file_name().unwrap().to_str().unwrap(),
            result
        );
    }

    println!(
        "\n{} out of {} fonts passed ({} failed)\n",
        (fonts.len() - num_errors),
        fonts.len(),
        num_errors
    );

    assert_eq!(num_errors, 0, "Not all font files parsed successfully");
}
