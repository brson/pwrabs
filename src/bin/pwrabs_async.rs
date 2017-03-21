#![no_std]
#![feature(collections)]
#![feature(alloc)]
#![feature(box_patterns)]
#![feature(link_args)]

extern crate alloc;
#[macro_use] extern crate collections;
extern crate pwrabs;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate fst;
#[macro_use] extern crate wheel;
extern crate futures;

use alloc::boxed::Box;
use collections::Vec;
use fst::raw::Fst;
use futures::{Future, Stream, future, Sink};
use wheel::prelude::*;
use wheel::Port;
use pwrabs::{Config, TestSet};

#[cfg_attr(target_arch="asmjs",
    link_args="\
        -s INVOKE_RUN=0 \
        --js-library ../wheel/src/asmjs/src/ffi.js \
        --emit-symbol-map \
")]
extern {}

#[no_mangle]
pub extern "C" fn start(port: u32, dict: u32) {
    let dict = File::from_handle(dict);
    let port = Port::from_handle(port);
    let f = dict.read().map_err(|e| format!("failed to read dict: {:?}", e))
    .and_then(|data|
        future::result(Fst::from_bytes(data))
        .map_err(|e| format!("failed to read dict: {:?}", e))
    )
    .and_then(|fst| {
        let config = Config::new(fst);
        let (sink, stream) = port.split();
        
        stream.map(move |data| {
            match serde_json::from_slice(&data) {
                Ok(set) => match config.validate(&set) {
                    Ok(_) => "".into(),
                    Err(e) => serde_json::to_string(&e).unwrap()
                },
                Err(e) => format!("failed to parse: {:?}", e)
            }
        })
        .fold(sink, |sink, message| sink.send(message.into_bytes()))
    })
    .map(|_| 42);
    
    wheel::run(f);
}

fn main() {}
