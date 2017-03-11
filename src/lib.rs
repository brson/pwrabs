/// Password Rules are Bullshit
///
/// https://blog.codinghorror.com/password-rules-are-bullshit/
///
/// https://www.reddit.com/r/programming/comments/5ym1fv/password_rules_are_bullshit/
///
/// https://github.com/danielmiessler/SecLists/tree/master/Passwords
///

extern crate fst;
extern crate unicode_segmentation;

use fst::{IntoStreamer, Streamer, Set, SetBuilder};
use unicode_segmentation::UnicodeSegmentation;
use std::path::{PathBuf};

// NB: This number must also be changed in build.rs
const DEFAULT_MIN_GLYPHS: u32 = 10;
const DEFAULT_MAX_BYTES: u32 = 1024;
const DEFAULT_UNIQUE_GLYPHS: u32 = 5;

pub fn pwrabs(pw: &str, username: &str, email: &str) -> Result<(), Error> {
    Config::new(username, email).validate(pw)
}

pub struct Config<'a> {
    /// Passwords must contain a minimum number of glyphs
    min_glyphs: u32,
    /// Some of the glyphs must be unique
    unique_glyphs: u32,
    /// Passwords must fit into a reasonable number of bytes
    max_bytes: u32,
    /// Can't user username as password
    username: &'a str,
    /// Nor email
    email: &'a str,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
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

impl<'a> Config<'a> {
    fn new(username: &'a str, email: &'a str) -> Config<'a> {
        Config {
            min_glyphs: DEFAULT_MIN_GLYPHS,
            unique_glyphs: DEFAULT_UNIQUE_GLYPHS,
            max_bytes: DEFAULT_MAX_BYTES,
            username: username,
            email: email,
        }
    }

    fn validate(&self, pw: &str) -> Result<(), Error> {
        // Do simple validation
        self.validate_max_bytes(pw)?;
        self.validate_min_and_unique_glyphs(pw)?;
        self.validate_username(pw)?;
        self.validate_email(pw)?;

        // Check the common passwords FST
        self.validate_common_passwords(pw)?;

        Ok(())
    }

    fn validate_max_bytes(&self, pw: &str) -> Result<(), Error> {
        if pw.len() > self.max_bytes as usize {
            Err(Error::MaxBytes(pw.len(), self.max_bytes))
        } else {
            Ok(())
        }
    }

    fn validate_min_and_unique_glyphs(&self, pw: &str) -> Result<(), Error> {
        use std::collections::HashSet;
        // Allocate a hash set to track unique graphemes
        let mut glyphs = HashSet::with_capacity(self.unique_glyphs as usize);
        let mut total = 0;
        // Parse the graphemes out of the password
        let graphemes = pw.graphemes(true);
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

    fn validate_username(&self, pw: &str) -> Result<(), Error> {
        if pw == self.username {
            Err(Error::Username(self.username.to_string()))
        } else {
            Ok(())
        }
    }

    fn validate_email(&self, pw: &str) -> Result<(), Error> {
        if pw == self.email {
            Err(Error::Email(self.email.to_string()))
        } else {
            Ok(())
        }
    }

    fn validate_common_passwords(&self, pw: &str) -> Result<(), Error> {
        let pwfst = include_bytes!(concat!(env!("OUT_DIR"), "/pws.fst"));
        // FIXME: Don't allocate vectors
        let pwset = Set::from_bytes(pwfst.to_vec()).expect("");
        if pwset.contains(pw) {
            Err(Error::Common(pw.to_string()))
        } else {
            Ok(())
        }
    }
}

// The C interface
mod cc {
    #[no_mangle]
    extern fn pwrabs(pw: *const u8,
                     username: *const u8,
                     email: *const u8) -> *const u8
    {
        panic!()
    }

    #[no_mangle]
    extern fn pwrabs_free(res: *const u8) {
        panic!()
    }
}

