// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use context::GraphicsContextMethods;

use libc::{c_char, c_int, c_long, c_uint, c_ulong, c_void};
use std::ptr;
use sync::Arc;

// Constants.

static GLX_RGBA: c_int = 4;
/*static GLX_RED_SIZE: c_int = 8;
static GLX_GREEN_SIZE: c_int = 9;
static GLX_BLUE_SIZE: c_int = 10;*/
static GLX_DEPTH_SIZE: c_int = 12;

static mut ATTRIBUTES: [c_int, ..4] = [
    GLX_RGBA,
    GLX_DEPTH_SIZE, 24,
    0,
];

// External bindings to Xlib.

// Opaque structures.
struct _XPrivate;
struct _XrmHashBucketRec;
struct Depth;
struct GLXContextOpaque;
struct Visual;

// Sadly we need to copy some of this definition in here because the Xlib macros need to access it.
struct Display {
    _ext_data: *mut XExtData,
    _private1: *mut _XPrivate,
    _fd: c_int,
    _private2: c_int,
    _proto_major_version: c_int,
    _proto_minor_version: c_int,
    _vendor: *mut c_char,
    _private3: XID,
    _private4: XID,
    _private5: XID,
    _private6: c_int,
    _resource_alloc: *mut c_void,
    _byte_order: c_int,
    _bitmap_unit: c_int,
    _bitmap_pad: c_int,
    _bitmap_bit_order: c_int,
    _nformats: c_int,
    _pixmap_format: *mut ScreenFormat,
    _private8: c_int,
    _release: c_int,
    _private9: *mut _XPrivate,
    _private10: *mut _XPrivate,
    _qlen: c_int,
    _last_request_read: c_ulong,
    _request: c_ulong,
    _private11: XPointer,
    _private12: XPointer,
    _private13: XPointer,
    _private14: XPointer,
    _max_request_size: c_uint,
    _db: *mut _XrmHashBucketRec,
    _private15: *mut c_void,
    _display_name: *mut c_char,

    // FIXME(pcwalton): These are needed for some reason...
    _pad: *mut c_void,
    _pad1: *mut c_void,

    default_screen: c_int,
    _nscreens: c_int,
    screens: *mut Screen,
}

struct Screen {
    _ext_data: *mut XExtData,
    _display: *mut Display,
    root: Window,
    _width: c_int,
    _height: c_int,
    _mwidth: c_int,
    _mheight: c_int,
    _ndepths: c_int,
    _depths: *mut Depth,
    _root_depth: c_int,
    _root_visual: *mut Visual,
    _default_gc: GC,
    _cmap: Colormap,
    _white_pixel: c_ulong,
    _black_pixel: c_ulong,
    _max_maps: c_int,
    _min_maps: c_int,
    _backing_store: c_int,
    _save_unders: Bool,
    _root_input_mask: c_long,
    _pad: *mut c_void,
}

struct ScreenFormat;
struct XExtData;

struct XVisualInfo {
    _visual: *mut Visual,
    _visualid: VisualID,
    _screen: c_int,
    _depth: c_int,
    _class: c_int,
    _red_mask: c_ulong,
    _green_mask: c_ulong,
    _blue_mask: c_ulong,
    _colormap_size: c_int,
    _bits_per_rgb: c_int,
}

type Bool = c_int;
type Colormap = XID;
type Drawable = c_uint;                 // compatible with Window
type GC = *mut c_void;
pub type GLXContext = *mut GLXContextOpaque;
type GLXDrawable = c_uint;              // compatible with GLXPixmap
type GLXPixmap = c_uint;                // compatible with GLXDrawable
type Pixmap = c_uint;
type VisualID = c_ulong;
type Window = c_uint;                   // compatible with Drawable
type XID = c_uint;
type XPointer = *mut c_void;

#[link(name = "X11")]
#[link(name = "GL")]
extern {
    fn XOpenDisplay(n: c_int) -> *mut Display;
    fn XCreatePixmap(display: *mut Display, d: Drawable, width: c_uint, height: c_uint, depth: c_uint)
                     -> Pixmap;

    fn glXChooseVisual(dpy: *mut Display, screen: c_int, attribList: *mut c_int) -> *mut XVisualInfo;
    fn glXCreateContext(dpy: *mut Display, vis: *mut XVisualInfo, shareList: GLXContext, direct: Bool)
                        -> GLXContext;
    fn glXCreateGLXPixmap(dpy: *mut Display, vis: *mut XVisualInfo, pixmap: Pixmap) -> GLXPixmap;
    fn glXMakeContextCurrent(dpy: *mut Display, draw: GLXDrawable, read: GLXDrawable, ctx: GLXContext)
                             -> Bool;
}

// X11 macros

fn DefaultScreen(dpy: *mut Display) -> c_int {
    unsafe {
        (*dpy).default_screen
    }
}
fn RootWindow(dpy: *mut Display, scr: c_int) -> Window {
    unsafe {
        (*ScreenOfDisplay(dpy, scr)).root
    }
}
fn ScreenOfDisplay(dpy: *mut Display, scr: c_int) -> *mut Screen {
    unsafe {
        (&(*dpy).screens).offset(scr as int)
    }
}

// Implementation

/// Linux-specific interface to 3D graphics contexts.
pub struct GraphicsContext {
    display: *mut Display,
    pixmap: GLXPixmap,
    context: Arc<GLXContext>,
}

impl GraphicsContext {
    // Creates a new, possibly shared, GLX context.
    fn new_possibly_shared(share_context: Option<GraphicsContext>) -> GraphicsContext {
        let (display, visual, pixmap) = GraphicsContext::create_display_visual_and_pixmap();

        unsafe {
            let context = match share_context {
                None => glXCreateContext(display, visual, ptr::mut_null(), 1),
                Some(share_context) => {
                    let native_share_context = share_context.native();
                    glXCreateContext(display, visual, *native_share_context, 1)
                }
            };

            assert!(context != ptr::mut_null());

            GraphicsContext {
                display: display,
                pixmap: pixmap,
                context: Arc::new(context),
            }
        }
    }

    fn create_display_visual_and_pixmap() -> (*mut Display, *mut XVisualInfo, GLXPixmap) {
        unsafe {
            // Get a connection.
            let display = XOpenDisplay(0);

            // Get an appropriate visual.
            let visual = glXChooseVisual(display, DefaultScreen(display), &mut ATTRIBUTES[0]);

            // Create the pixmap.
            let root_window = RootWindow(display, DefaultScreen(display));
            let pixmap = XCreatePixmap(display, root_window, 10, 10, 24);
            let glx_pixmap = glXCreateGLXPixmap(display, visual, pixmap);

            debug!("XCreatePixmap returned {}, glXCreateGLXPixmap returned {}",
                   pixmap,
                   glx_pixmap);

            (display, visual, glx_pixmap)
        }
    }
}

impl GraphicsContextMethods<GLXContext> for GraphicsContext {
    /// Wraps the given instance of the native GLX graphics context, bumping the reference count in
    /// the process.
    fn wrap(instance: Arc<GLXContext>) -> GraphicsContext {
        let (display, _, pixmap) = GraphicsContext::create_display_visual_and_pixmap();
        GraphicsContext {
            display: display,
            pixmap: pixmap,
            context: instance.clone(),
        }
    }

    /// Returns the underlying native 3D context.
    fn native(&self) -> Arc<GLXContext> {
        self.context.clone()
    }

    /// Creates a new offscreen 3D graphics context.
    fn new() -> GraphicsContext {
        GraphicsContext::new_possibly_shared(None)
    }

    /// Creates a new offscreen 3D graphics context shared with the given context.
    fn new_shared(share_context: GraphicsContext) -> GraphicsContext {
        GraphicsContext::new_possibly_shared(Some(share_context))
    }

    /// Makes this context the current context.
    fn make_current(&self) {
        unsafe {
            let result = glXMakeContextCurrent(self.display,
                                               self.pixmap,
                                               self.pixmap,
                                               *self.context);
            assert!(result != 0);
        }
    }
}

impl Clone for GraphicsContext {
    fn clone(&self) -> GraphicsContext {
        GraphicsContext {
            display: self.display,
            pixmap: self.pixmap,
            context: self.context.clone(),
        }
    }
}

