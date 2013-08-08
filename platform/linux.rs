// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use context::GraphicsContextMethods;

use std::libc::{c_char, c_int, c_long, c_uint, c_ulong, c_void};
use std::ptr::null;
use std::ptr;
use extra::arc::Arc;

// Constants.

static GLX_RGBA: c_int = 4;
static GLX_RED_SIZE: c_int = 8;
static GLX_GREEN_SIZE: c_int = 9;
static GLX_BLUE_SIZE: c_int = 10;
static GLX_DEPTH_SIZE: c_int = 12;

static ATTRIBUTES: [c_int, ..4] = [
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
    ext_data: *XExtData,
    private1: *_XPrivate,
    fd: c_int,
    private2: c_int,
    proto_major_version: c_int,
    proto_minor_version: c_int,
    vendor: *c_char,
    private3: XID,
    private4: XID,
    private5: XID,
    private6: c_int,
    resource_alloc: *c_void,
    byte_order: c_int,
    bitmap_unit: c_int,
    bitmap_pad: c_int,
    bitmap_bit_order: c_int,
    nformats: c_int,
    pixmap_format: *ScreenFormat,
    private8: c_int,
    release: c_int,
    private9: *_XPrivate,
    private10: *_XPrivate,
    qlen: c_int,
    last_request_read: c_ulong,
    request: c_ulong,
    private11: XPointer,
    private12: XPointer,
    private13: XPointer,
    private14: XPointer,
    max_request_size: c_uint,
    db: *_XrmHashBucketRec,
    private15: *c_void,
    display_name: *c_char,

    // FIXME(pcwalton): These are needed for some reason...
    pad: *c_void,
    pad1: *c_void,

    default_screen: c_int,
    nscreens: c_int,
    screens: *Screen,
}

struct Screen {
    ext_data: *XExtData,
    display: *Display,
    root: Window,
    width: c_int,
    height: c_int,
    mwidth: c_int,
    mheight: c_int,
    ndepths: c_int,
    depths: *Depth,
    root_depth: c_int,
    root_visual: *Visual,
    default_gc: GC,
    cmap: Colormap,
    white_pixel: c_ulong,
    black_pixel: c_ulong,
    max_maps: c_int,
    min_maps: c_int,
    backing_store: c_int,
    save_unders: Bool,
    root_input_mask: c_long,
    pad: *c_void,
}

struct ScreenFormat;
struct XExtData;

struct XVisualInfo {
    visual: *Visual,
    visualid: VisualID,
    screen: c_int,
    depth: c_int,
    class: c_int,
    red_mask: c_ulong,
    green_mask: c_ulong,
    blue_mask: c_ulong,
    colormap_size: c_int,
    bits_per_rgb: c_int,
}

type Bool = c_int;
type Colormap = XID;
type Drawable = c_uint;                 // compatible with Window
type GC = *c_void;
type GLXContext = *GLXContextOpaque;
type GLXDrawable = c_uint;              // compatible with GLXPixmap
type GLXPixmap = c_uint;                // compatible with GLXDrawable
type Pixmap = c_uint;
type VisualID = c_ulong;
type Window = c_uint;                   // compatible with Drawable
type XID = c_uint;
type XPointer = *c_void;

#[link_args="-lX11"]
#[nolink]
extern {
    fn XOpenDisplay(n: c_int) -> *Display;
    fn XCreatePixmap(display: *Display, d: Drawable, width: c_uint, height: c_uint, depth: c_uint)
                     -> Pixmap;

    fn glXChooseVisual(dpy: *Display, screen: c_int, attribList: *c_int) -> *XVisualInfo;
    fn glXCreateContext(dpy: *Display, vis: *XVisualInfo, shareList: GLXContext, direct: Bool)
                        -> GLXContext;
    fn glXCreateGLXPixmap(dpy: *Display, vis: *XVisualInfo, pixmap: Pixmap) -> GLXPixmap;
    fn glXMakeContextCurrent(dpy: *Display, draw: GLXDrawable, read: GLXDrawable, ctx: GLXContext)
                             -> Bool;
}

// X11 macros

fn DefaultScreen(dpy: *Display) -> c_int {
    unsafe {
        (*dpy).default_screen
    }
}
fn RootWindow(dpy: *Display, scr: c_int) -> Window {
    unsafe {
        (*ScreenOfDisplay(dpy, scr)).root
    }
}
fn ScreenOfDisplay(dpy: *Display, scr: c_int) -> *Screen {
    unsafe {
        *ptr::offset(&(*dpy).screens, scr as int)
    }
}

// Implementation

/// Linux-specific interface to 3D graphics contexts.
pub struct GraphicsContext {
    priv display: *Display,
    priv pixmap: GLXPixmap,
    priv context: Arc<GLXContext>,
}

impl GraphicsContext {
    // Creates a new, possibly shared, GLX context.
    fn new_possibly_shared(share_context: Option<GraphicsContext>) -> GraphicsContext {
        let (display, visual, pixmap) = GraphicsContext::create_display_visual_and_pixmap();

        unsafe {
            let context = match share_context {
                None => glXCreateContext(display, visual, null(), 1),
                Some(share_context) => {
                    let native_share_context = share_context.native();
                    glXCreateContext(display, visual, *native_share_context.get(), 1)
                }
            };

            assert!(context != null());

            GraphicsContext {
                display: display,
                pixmap: pixmap,
                context: Arc::new(context),
            }
        }
    }

    fn create_display_visual_and_pixmap() -> (*Display, *XVisualInfo, GLXPixmap) {
        unsafe {
            // Get a connection.
            let display = XOpenDisplay(0);

            // Get an appropriate visual.
            let visual = glXChooseVisual(display, DefaultScreen(display), &ATTRIBUTES[0]);

            // Create the pixmap.
            let root_window = RootWindow(display, DefaultScreen(display));
            let pixmap = XCreatePixmap(display, root_window, 10, 10, 24);
            let glx_pixmap = glXCreateGLXPixmap(display, visual, pixmap);

            debug!("XCreatePixmap returned %?, glXCreateGLXPixmap returned %?",
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
                                               *self.context.get());
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

