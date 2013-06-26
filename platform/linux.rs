// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::cell::Cell;
use core::libc::{c_char, c_int, c_uint, c_ulong, c_void};
use core::ptr::null;
use std::arc::ARC;

// Constants.

static GLX_RGBA: c_int = 4;
static GLX_RED_SIZE: c_int = 8;
static GLX_GREEN_SIZE: c_int = 9;
static GLX_BLUE_SIZE: c_int = 10;

static ATTRIBUTES: [c_int, ..8] = [
    GLX_RGBA,
    GLX_RED_SIZE, 1, 
    GLX_GREEN_SIZE, 1,
    GLX_BLUE_SIZE, 1,
    0,
];

// External bindings to Xlib.

struct _XPrivate;
struct _XrmHashBucketRec;
struct Depth;

// Sadly we need to copy some of this definition in here because the Xlib macros need to access it.
struct Display {
    ext_data: *XExtData;
    private1: *_XPrivate;
    fd: c_int,
    private2: c_int,
    proto_major_version: c_int,
    proto_minor_version: c_int,
    vendor: *c_char,
    private3: XID,
    private4: XID,
    private5: XID,
    private6: XID,
    resource_alloc: *c_void,
    byte_order: c_int,
    bitmap_unit: c_int,
    bitmap_pad: c_int,
    bitmap_bit_order: c_int,
    nformats: c_int,
    pixmap_format: *ScreenFormat,
    private8: c_int,
    release: c_int,
    private9: *XPrivate,
    private10: *XPrivate,
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
    default_screen: c_int,
    nscreens: c_int,
    screens: *Screen,
}

struct GLXContextOpaque;

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
type XID = c_ulong;
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
    fn glXMakeCurrent(dpy: *Display, drawable: GLXDrawable, context: GLXContext) -> Bool;
}

// X11 macros

fn DefaultScreen(dpy: *Display) -> *Screen {
    unsafe {
        dpy.default_screen
    }
}
fn RootWindow(dpy: *Display, scr: c_int) -> Window {
    ScreenOfDisplay(dpy, scr).root
}
fn ScreenOfDisplay(dpy: *Display, scr: c_int) -> *Screen {
    unsafe {
        &dpy.screens[scr]
    }
}

// Implementation

/// Linux-specific interface to 3D graphics contexts.
pub struct GraphicsContext {
    priv display: *Display,
    priv pixmap: GLXPixmap,
    priv context: GLXContext,
}

impl GraphicsContext {
    fn create_display_and_pixmap() -> (Display, GLXPixmap) {
        unsafe {
            // Get a connection.
            let display = XOpenDisplay(0);

            // Get an appropriate visual.
            let visual = glXChooseVisual(display, DefaultScreen(display), &ATTRIBUTES[0]);
            assert!(visual != null());

            // Create the pixmap.
            let root_window = RootWindow(display, visual.screen);
            let pixmap = XCreatePixmap(display, root_window, 500, 500, visual.depth);
            let glx_pixmap = glXCreateGLXPixmap(display, visual, pixmap);

            (display, glx_pixmap)
        }
    }
}

impl GraphicsContextMethods<ARC<GLXContext>> for GraphicsContext {
    /// Returns the current graphics context.
    ///
    /// X11 does not seem to support this functionality, so this fails.
    fn current() -> GraphicsContext {
        fail!("GraphicsContext::current() unsupported on X11")
    }

    /// Wraps the given instance of the native GLX graphics context, bumping the reference count in
    /// the process.
    fn wrap(instance: ARC<GLXContext>) -> GraphicsContext {
        let (display, pixmap) = GraphicsContext::create_display_and_pixmap();
        GraphicsContext {
            display: display,
            pixmap: pixmap,
            context: instance.clone(),
        }
    }

    /// Returns the underlying native 3D context.
    fn native(&self) -> ARC<GLXContext> {
        self.context.clone()
    }

    /// Creates a new offscreen 3D graphics context.
    fn new() -> GraphicsContext {
        let (display, pixmap) = GraphicsContext::create_display_and_pixmap();

        unsafe {
            let context = glXCreateContext(display, visual, 0, 0);
            assert!(context != null());

            GraphicsContext {
                display: display,
                pixmap: pixmap,
                context: ARC(context),
            }
        }
    }

    /// Makes this context the current context.
    fn make_current(&self) {
        unsafe {
            let result = glXMakeCurrent(self.display, self.pixmap, self.context);
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

