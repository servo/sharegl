// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A platform-independent interface to 3D graphics contexts.

use std::arc::ARC;

/// Platform-independent interface to 3D graphics contexts.
pub trait GraphicsContextMethods<NativeContextType> {
    /// Wraps the given instance of the native 3D context, incrementing its reference count.
    fn wrap(instance: ARC<NativeContextType>) -> Self;

    /// Returns the underlying native 3D context.
    fn native(&self) -> ARC<NativeContextType>;

    /// Creates a new offscreen 3D graphics context.
    fn new() -> Self;

    /// Creates a new offscreen 3D graphics context shared with the given context.
    fn new_shared(share_context: Self) -> Self;

    /// Makes this context the current context, so that all graphics operations will go here.
    fn make_current(&self);
}

