extern crate fst;
extern crate unicode_segmentation;

use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::env;
use std::path::PathBuf;
use std::collections::BTreeSet;

use fst::SetBuilder;
use unicode_segmentation::UnicodeSegmentation;

// The number of passwords to include in the FST,
// There are 100,000 in the list of the top 1M.
const PW_SET_SIZE: usize = 100_000;

fn main() {
    let pwdir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let pwdir = PathBuf::from(pwdir);
    let pwfile = pwdir.join("pws.txt");
    let pwfile = File::open(pwfile).expect("open pws.txt");
    let pwfile = BufReader::new(pwfile);

    let mut pws = BTreeSet::new();
    for line in pwfile.lines() {
        let line = line.unwrap();
        let glyphs = line.graphemes(true).count();
        // If it's a short password then it will be detected trivially.
        // Don't put it in the fst.
        // if glyphs < DEFAULT_MIN_GLYPHS { continue }
        pws.insert(line);

        if pws.len() >= PW_SET_SIZE {
            break;
        }
    }

    let mut build = SetBuilder::memory();
    for pw in pws {
        build.insert(pw).unwrap();
    }
    let fst = build.into_inner().unwrap();

    let outdir = env::var("OUT_DIR").unwrap();
    let outdir = PathBuf::from(outdir);
    let pwset = outdir.join("pws.fst");
    let mut pwset = File::create(pwset).expect("create pws.fst");
    pwset.write_all(&fst).unwrap();
}
