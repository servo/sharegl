// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use base::ShareContext;
use context::GraphicsContextMethods;

use core::cast::transmute;
use core::ptr::{null, to_unsafe_ptr};
use geom::size::Size2D;
use io_surface::{IOSurface, kIOSurfaceBytesPerElement, kIOSurfaceBytesPerRow};
use io_surface::{kIOSurfaceHeight, kIOSurfaceIsGlobal, kIOSurfaceWidth};
use io_surface::IOSurfaceMethods;
use opengles::cgl::{CGLChoosePixelFormat, CGLContextObj, CGLCreateContext, CGLReleaseContext};
use opengles::cgl::{CGLSetCurrentContext, CGLTexImageIOSurface2D, kCGLNoError, kCGLPFACompliant};
use opengles::cgl::{kCGLPFADoubleBuffer};
use opengles::gl2::{BGRA, CLAMP_TO_EDGE, COLOR_ATTACHMENT0, FRAMEBUFFER};
use opengles::gl2::{FRAMEBUFFER_COMPLETE, GLenum, GLint, GLsizei, GLuint, LINEAR};
use opengles::gl2::{NEAREST, RGBA, TEXTURE_MAG_FILTER, TEXTURE_MIN_FILTER};
use opengles::gl2::{TEXTURE_RECTANGLE_ARB, TEXTURE_WRAP_S, TEXTURE_WRAP_T};
use opengles::gl2::{UNSIGNED_INT_8_8_8_8_REV};
use opengles::gl2;
use std::arc::ARC;
use std::arc;

// FIXME: This is not good.
#[link_args="-framework IOSurface -framework CoreFoundation"]
#[nolink]
extern {}

/// Mac-specific interface to 3D graphics contexts.
pub struct GraphicsContext {
    cgl_context: ARC<CGLContextObj>,
}

impl GraphicsContext {
    /// Returns a new context, possibly shared with another context.
    fn new_possibly_shared(share_context: Option<GraphicsContext>) -> GraphicsContext {
        unsafe {
            // Choose a pixel format.
            let attributes = [ kCGLPFADoubleBuffer, kCGLPFACompliant, 0 ];
            let pixel_format = null();
            let pixel_format_count = 1;
            let gl_error = CGLChoosePixelFormat(transmute(&attributes[0]),
                                                to_unsafe_ptr(&pixel_format),
                                                to_unsafe_ptr(&pixel_format_count));
            assert!(gl_error == kCGLNoError);

            // Create the context.
            let cgl_context = null();
            let gl_error = match share_context {
                None => CGLCreateContext(pixel_format, null(), &cgl_context),
                Some(ref share_context) => {
                    let native = share_context.native();
                    CGLCreateContext(pixel_format, *arc::get(&native), &cgl_context)
                }
            };
            assert!(gl_error == kCGLNoError);

            GraphicsContextMethods::wrap(ARC(cgl_context))
        }
    }
}

impl GraphicsContextMethods<CGLContextObj> for GraphicsContext {
    /// Wraps the given instance of the native Core OpenGL graphics context.
    fn wrap(instance: ARC<CGLContextObj>) -> GraphicsContext {
        unsafe {
            GraphicsContext {
                cgl_context: instance
            }
        }
    }

    /// Returns the underlying native 3D context without modifying its reference count.
    fn native(&self) -> ARC<CGLContextObj> {
        self.cgl_context.clone()
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
            let gl_error = CGLSetCurrentContext(*arc::get(&self.cgl_context));
            assert!(gl_error == kCGLNoError)
        }
    }
}

impl Clone for GraphicsContext {
    fn clone(&self) -> GraphicsContext {
        GraphicsContextMethods::wrap(self.native())
    }
}

pub struct Context {
    surface: IOSurface,
    framebuffer: GLuint,
    texture: GLuint
    
    // FIXME: Needs drop.
}

pub fn init_cgl() -> GraphicsContext {
    let context: GraphicsContext = GraphicsContextMethods::new();
    context.make_current();
    context
}


pub fn init_surface(size: Size2D<int>) -> IOSurface {
    use core_foundation::boolean::CFBoolean;
    use core_foundation;
    use io_surface;
    use number = core_foundation::number::CFNumber::new;
    use string = core_foundation::string::CFString::wrap_shared;

    // TODO: dictionary constructor should be less ridiculous.
    // Or, we could add bindings for mutable dictionaries.
    let k_width = string(kIOSurfaceWidth);
    let v_width = number(size.width as i32);

    let k_height = string(kIOSurfaceHeight);
    let v_height = number(size.height as i32);

    let k_bytes_per_row = string(kIOSurfaceBytesPerRow);
    let v_bytes_per_row = number(size.width as i32 * 4);

    let k_bytes_per_elem = string(kIOSurfaceBytesPerElement);
    let v_bytes_per_elem = number(4i32);

    let k_is_global = string(kIOSurfaceIsGlobal);
    let v_is_global = CFBoolean::true_value();

    io_surface::new(&core_foundation::dictionary::CFDictionary::new([
        (*k_width.contents.borrow_ref(),          *v_width.contents.borrow_type_ref()),
        (*k_height.contents.borrow_ref(),         *v_height.contents.borrow_type_ref()),
        (*k_bytes_per_row.contents.borrow_ref(),  *v_bytes_per_row.contents.borrow_type_ref()),
        (*k_bytes_per_elem.contents.borrow_ref(), *v_bytes_per_elem.contents.borrow_type_ref()),
        (*k_is_global.contents.borrow_ref(),      *v_is_global.contents.borrow_type_ref()),
    ]))
}

pub fn init_texture() -> GLuint {
    gl2::enable(TEXTURE_RECTANGLE_ARB);

    let texture = gl2::gen_textures(1)[0];
    gl2::bind_texture(TEXTURE_RECTANGLE_ARB, texture);
    gl2::tex_parameter_i(TEXTURE_RECTANGLE_ARB, TEXTURE_WRAP_S, CLAMP_TO_EDGE as GLint);
    gl2::tex_parameter_i(TEXTURE_RECTANGLE_ARB, TEXTURE_WRAP_T, CLAMP_TO_EDGE as GLint);
    gl2::tex_parameter_i(TEXTURE_RECTANGLE_ARB, TEXTURE_MAG_FILTER, LINEAR as GLint);
    gl2::tex_parameter_i(TEXTURE_RECTANGLE_ARB, TEXTURE_MIN_FILTER, NEAREST as GLint);
    return texture;
}

// Assumes the texture is already bound via gl2::bind_texture().
pub fn bind_surface_to_texture(context: &GraphicsContext, surface: &IOSurface, size: Size2D<int>) {
    // FIXME: There should be safe wrappers for this.
    unsafe {
        let native = context.native();
        let gl_error = CGLTexImageIOSurface2D(*arc::get(&native),
                                              TEXTURE_RECTANGLE_ARB,
                                              RGBA as GLenum,
                                              size.width as GLsizei,
                                              size.height as GLsizei,
                                              BGRA as GLenum,
                                              UNSIGNED_INT_8_8_8_8_REV,
                                              transmute(copy surface.obj),
                                              0);
        assert!(gl_error == kCGLNoError);
    }
}

pub fn bind_texture_to_framebuffer(texture: GLuint) {
    gl2::bind_texture(TEXTURE_RECTANGLE_ARB, 0);
    gl2::framebuffer_texture_2d(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_RECTANGLE_ARB, texture, 0);
    assert!(gl2::check_framebuffer_status(FRAMEBUFFER) == FRAMEBUFFER_COMPLETE);
}

impl ShareContext for Context {
    fn new(size: Size2D<int>) -> Context {
        // Initialize CGL.
        let context = init_cgl();

        // Create the surface.
        let surface = init_surface(copy size);

        // Create a framebuffer.
        let framebuffer = gl2::gen_framebuffers(1)[0];
        gl2::bind_framebuffer(FRAMEBUFFER, framebuffer);

        // Create and bind to the texture.
        let texture = init_texture();
        bind_surface_to_texture(&context, &surface, size);

        // Bind the texture to the framebuffer.
        bind_texture_to_framebuffer(texture);

        Context {
            surface: surface,
            framebuffer: framebuffer,
            texture: texture
        }
    }

    fn flush(&self) {
        gl2::finish();
    }

    fn id(&self) -> int {
        self.surface.get_id() as int
    }
}

