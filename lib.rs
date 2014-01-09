// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[crate_id = "github.com/mozilla-servo/sharegl#0.1"];

extern mod std;
extern mod extra;
extern mod geom;

#[cfg(target_os="macos")]
extern mod core_foundation;
#[cfg(target_os="macos")]
extern mod io_surface;
#[cfg(target_os="macos")]
extern mod opengles;

pub mod base;
pub mod context;

#[cfg(target_os="macos")]
#[path="platform/macos.rs"]
pub mod platform;

#[cfg(target_os="linux")]
#[path="platform/linux.rs"]
pub mod platform;

#[cfg(target_os="windows")]
#[cfg(target_os="android")]
#[path="platform/dummy.rs"]
pub mod platform;
