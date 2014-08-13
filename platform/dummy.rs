// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use geom::size::Size2D;
use base::ShareContext;

pub type Context = DummyContext;

struct DummyContext {
    _unused: int
}

impl ShareContext for DummyContext {
    fn new(_size: Size2D<int>) -> DummyContext {
        DummyContext {
            _unused: 0
        }
    }

    fn flush(&self) {
    }

    fn id(&self) -> int {
        0
    }
}
