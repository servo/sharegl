// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use geom::size::Size2D;

pub trait ShareContext {
    // Creates a new context for GL object sharing.
    fn new(size: Size2D<int>) -> Self;

    // Flushes the context.
    fn flush(&self);

    // Returns the platform-specific ID that can be passed to other processes to access the shared
    // resources.
    fn id(&self) -> int;
}

#[test]
fn smoke() {}

