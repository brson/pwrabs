#![no_std]
#![feature(collections)]
#![feature(alloc)]
#![feature(box_patterns)]

/// Password Rules are Bullshit
///
/// https://blog.codinghorror.com/password-rules-are-bullshit/
///
/// https://www.reddit.com/r/programming/comments/5ym1fv/password_rules_are_bullshit/
///
/// https://github.com/danielmiessler/SecLists/tree/master/Passwords
///

extern crate collections;
#[macro_use] extern crate serde_derive;
extern crate fst;
extern crate unicode_segmentation;
extern crate serde_json;
extern crate serde;
extern crate alloc;

use collections::{Vec, BTreeSet, String};
use collections::string::ToString;
use fst::Set;
use fst::raw::Fst;
use unicode_segmentation::UnicodeSegmentation;

// NB: This number must also be changed in build.rs
const DEFAULT_MIN_GLYPHS: u32 = 8;
const DEFAULT_MAX_BYTES: u32 = 1024;
const DEFAULT_UNIQUE_GLYPHS: u32 = 5;

pub struct Config {
    /// Passwords must contain a minimum number of glyphs
    min_glyphs:     u32,
    /// Some of the glyphs must be unique
    unique_glyphs:  u32,
    /// Passwords must fit into a reasonable number of bytes
    max_bytes:      u32,
    
    passwords:      Set
}

#[derive(Deserialize)]
pub struct TestSet {
    password:   String,
    /// Can't user username as password
    username:   String,
    /// Nor email
    email:      String,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
#[derive(Serialize)]
pub enum Error {
    /// actual, min
    MinGlyphs(usize, u32),
    /// actual, max
    MaxBytes(usize, u32),
    /// unique, min unique
    UniqueGlyphs(usize, u32),
    Username(String),
    Email(String),
    /// Common password
    Common(String),
}

impl Config {
    pub fn new(fst: Fst) -> Config {
        Config {
            min_glyphs:     DEFAULT_MIN_GLYPHS,
            unique_glyphs:  DEFAULT_UNIQUE_GLYPHS,
            max_bytes:      DEFAULT_MAX_BYTES,
            passwords:      Set::from(fst)
        }
    }
    pub fn validate(&self, set: &TestSet) -> Result<(), Error> {
        // Do simple validation
        self.validate_max_bytes(set)?;
        self.validate_min_and_unique_glyphs(set)?;
        self.validate_username(set)?;
        self.validate_email(set)?;

        // Check the common passwords FST
        self.validate_common_passwords(set)?;

        Ok(())
    }

    fn validate_max_bytes(&self, set: &TestSet) -> Result<(), Error> {
        if set.password.len() > self.max_bytes as usize {
            Err(Error::MaxBytes(set.password.len(), self.max_bytes))
        } else {
            Ok(())
        }
    }

    fn validate_min_and_unique_glyphs(&self, set: &TestSet) -> Result<(), Error> {
        // Allocate a hash set to track unique graphemes
        let mut glyphs = BTreeSet::new();
        let mut total = 0;
        // Parse the graphemes out of the password
        let graphemes = set.password.graphemes(true);
        // But only parse as many as we need to
        let graphemes = graphemes.take(self.min_glyphs as usize);
        for glyph in graphemes {
            // Until we've seen as many as we need, store them in the set
            if glyphs.len() < self.unique_glyphs as usize {
                glyphs.insert(glyph);
            }
            total += 1;
        }

        if total < self.min_glyphs as usize {
            Err(Error::MinGlyphs(total, self.min_glyphs))
        } else if glyphs.len() < self.unique_glyphs as usize {
            Err(Error::UniqueGlyphs(glyphs.len(), self.unique_glyphs))
        } else {
            Ok(())
        }
    }

    fn validate_username(&self, set: &TestSet) -> Result<(), Error> {
        if set.password == set.username {
            Err(Error::Username(set.username.to_string()))
        } else {
            Ok(())
        }
    }

    fn validate_email(&self, set: &TestSet) -> Result<(), Error> {
        if set.password == set.email {
            Err(Error::Email(set.email.to_string()))
        } else {
            Ok(())
        }
    }

    fn validate_common_passwords(&self, set: &TestSet) -> Result<(), Error> {
        if self.passwords.contains(&set.password) {
            Err(Error::Common(set.password.to_string()))
        } else {
            Ok(())
        }
    }
}

pub struct Verifier {
    config: Config,
    buffer: Vec<u8>
}
impl Verifier {
    pub fn new(fst: Fst) -> Verifier {
        let config = Config::new(fst);
        Verifier {
            config: config,
            buffer: Vec::new()
        }
    }
    pub fn check(&mut self, set: &[u8]) -> Option<&[u8]> {
        self.buffer.clear();
        let set: TestSet = serde_json::from_slice(set).expect("failed to parse json");
        match self.config.validate(&set) {
            Ok(()) => None,
            Err(e) => {
                serde_json::to_writer(&mut self.buffer, &e).expect("failed to serialize");
                self.buffer.push(0);
                Some(&self.buffer)
            }
        }
    }
}

