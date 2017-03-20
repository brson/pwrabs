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

#[macro_use] extern crate collections;
#[macro_use] extern crate serde_derive;
extern crate fst;
extern crate unicode_segmentation;
extern crate serde_json;
extern crate serde;
extern crate alloc;

use alloc::boxed::Box;
use collections::{Vec, BTreeSet, String};
use collections::string::ToString;
use fst::{IntoStreamer, Streamer, Set, SetBuilder};
use fst::raw::Fst;
use unicode_segmentation::UnicodeSegmentation;

// NB: This number must also be changed in build.rs
const DEFAULT_MIN_GLYPHS: u32 = 10;
const DEFAULT_MAX_BYTES: u32 = 1024;
const DEFAULT_UNIQUE_GLYPHS: u32 = 5;

pub fn pwrabs(pw: &str, username: &str, email: &str) -> Result<(), Error> {
    Config::with_default(username, email).validate(pw)
}

#[derive(Deserialize)]
pub struct Config {
    /// Passwords must contain a minimum number of glyphs
    min_glyphs: u32,
    /// Some of the glyphs must be unique
    unique_glyphs: u32,
    /// Passwords must fit into a reasonable number of bytes
    max_bytes: u32,
    /// Can't user username as password
    username: String,
    /// Nor email
    email: String,
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
    fn with_default(username: &str, email: &str) -> Config {
        Config {
            min_glyphs: DEFAULT_MIN_GLYPHS,
            unique_glyphs: DEFAULT_UNIQUE_GLYPHS,
            max_bytes: DEFAULT_MAX_BYTES,
            username: username.to_string(),
            email: email.to_string(),
        }
    }
    
    fn from_json(json: &[u8]) -> Config {
        serde_json::from_slice(json).expect("invalid Config")
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
        // Allocate a hash set to track unique graphemes
        let mut glyphs = BTreeSet::new();
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
        let pwmap = Fst::from_static_slice(pwfst).expect("");
        
        let pwset = Set::from(pwmap);
        
        if pwset.contains(pw) {
            Err(Error::Common(pw.to_string()))
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
    fn from_config(config_buf: Vec<u8>) -> Verifier {
        let config = Config::from_json(&config_buf);
        Verifier {
            config: config,
            buffer: config_buf
        }
    }
    fn check(&mut self, password: &str) -> Option<&[u8]> {
        self.buffer.clear();
        match self.config.validate(password) {
            Ok(()) => None,
            Err(e) => {
                serde_json::to_writer(&mut self.buffer, &e);
                self.buffer.push(0);
                Some(&self.buffer)
            }
        }
    }
}

// The C interface
mod cc {
    use collections::Vec;
    use alloc::boxed::Box;
    pub use Verifier;
    use core;
    
    #[no_mangle]
    pub extern fn buf_create() -> *mut Vec<u8> {
        Box::into_raw(Box::new(Vec::new()))
    }
    #[no_mangle]
    pub extern fn buf_destroy(buf: *mut Vec<u8>) {
        unsafe { Box::from_raw(buf) };
    }
    
    #[no_mangle]
    pub extern fn buf_write(buf: *mut Vec<u8>, len: usize) -> *mut u8 {
        let buf = unsafe { &mut *buf };
        let cap = buf.capacity();
        if len > cap {
            buf.reserve(len - cap);
        }
        unsafe { buf.set_len(len) };
        buf.as_mut_ptr()
    }
    
    #[no_mangle]
    pub extern fn pwrabs_create(config: *mut Vec<u8>) -> *mut Verifier
    {
        let box config = unsafe { Box::from_raw(config) };
        let verifier = Verifier::from_config(config);
        Box::into_raw(Box::new(verifier))
    }

    #[no_mangle]
    pub extern fn pwrabs_verify(verifier: *mut Verifier, pass: *const Vec<u8>) -> *const u8 {
        use core::ptr;
        
        let verifier = unsafe { &mut *verifier };
        let pass = core::str::from_utf8(unsafe { &*pass }).expect("invalid utf8");
        
        match verifier.check(pass) {
            Some(err) => err.as_ptr(),
            None => ptr::null()
        }
    }
    
    #[no_mangle]
    pub extern fn pwrabs_free(verifier: *mut Verifier) {
        drop( unsafe { Box::from_raw(verifier) } );
    }
}

