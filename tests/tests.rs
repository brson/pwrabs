extern crate pwrabs;

use pwrabs::*;

#[test]
fn min_glyphs_err() {
    let err = pwrabs("abc", "", "").unwrap_err();
    assert_eq!(err, Error::MinGlyphs(3, 10));
}

#[test]
fn min_glyphs_ok() {
    let _ = pwrabs("abcdeffghijkl", "", "").unwrap();
}

#[test]
fn unique_glyphs_err() {
    let err = pwrabs("aaaaaaansr", "", "").unwrap_err();
    assert_eq!(err, Error::UniqueGlyphs(4, 5));
}

#[test]
fn unique_glyphs_ok() {
    let _ = pwrabs("aaaaabtnsr", "", "").unwrap();
}

#[test]
fn username() {
    let err = pwrabs("abrahamlincoln", "abrahamlincoln", "").unwrap_err();
    assert_eq!(err, Error::Username("abrahamlincoln".to_string()));
}

#[test]
fn email() {
    let err = pwrabs("abrahamlincoln@gmail.com", "", "abrahamlincoln@gmail.com").unwrap_err();
    assert_eq!(err, Error::Email("abrahamlincoln@gmail.com".to_string()));
}

#[test]
fn common_passwords() {
    let err = pwrabs("1234567890", "", "").unwrap_err();
    assert_eq!(err, Error::Common("1234567890".to_string()));
    let err = pwrabs("1234567890", "", "").unwrap_err();
    assert_eq!(err, Error::Common("1234567890".to_string()));
}

