extern mod core_foundation;
extern mod io_surface;
extern mod opengles;

use mod opengles::cgl;
use mod opengles::gl2;

use core_foundation::base::CFTypeOps;
use core::cast::transmute;
use core::ptr::{null, to_unsafe_ptr};
use geom::size::Size2D;
use io_surface::{IOSurface, kIOSurfaceBytesPerElement, kIOSurfaceBytesPerRow, kIOSurfaceHeight};
use io_surface::{kIOSurfaceIsGlobal, kIOSurfaceWidth};
use opengles::cgl::{CGLChoosePixelFormat, CGLContextObj, CGLCreateContext, CGLLockContext};
use opengles::cgl::{CGLSetCurrentContext, CGLTexImageIOSurface2D, kCGLNoError, kCGLPFACompliant};
use opengles::cgl::{kCGLPFADoubleBuffer};
use opengles::gl2::{BGRA, CLAMP_TO_EDGE, COLOR_ATTACHMENT0, FRAMEBUFFER, FRAMEBUFFER_COMPLETE};
use opengles::gl2::{GLenum, GLint, GLsizei, GLuint, LINEAR, NEAREST, RGBA, TEXTURE_MAG_FILTER};
use opengles::gl2::{TEXTURE_MIN_FILTER, TEXTURE_RECTANGLE_ARB, TEXTURE_WRAP_S, TEXTURE_WRAP_T};
use opengles::gl2::{UNSIGNED_INT_8_8_8_8_REV};

use base::ShareContext;

// FIXME: This is not good.
#[link_args="-framework IOSurface -framework CoreFoundation"]
#[nolink]
extern {}

pub type Context = MacContext;

pub struct MacContext {
    surface: IOSurface,
    framebuffer: GLuint,
    texture: GLuint
    
    // FIXME: Needs drop.
}

pub fn init_cgl() -> CGLContextObj {
    // FIXME: We should expose some higher-level, safe abstractions inside the CGL module.
    unsafe {
        // Choose a pixel format.
        let attributes = [ kCGLPFADoubleBuffer, kCGLPFACompliant, 0 ];
        let pixel_format = null();
        let pixel_format_count = 1;
        let gl_error = CGLChoosePixelFormat(transmute(&attributes[0]),
                                            to_unsafe_ptr(&pixel_format),
                                            to_unsafe_ptr(&pixel_format_count));
        assert gl_error == kCGLNoError;

        // Create the context.
        let cgl_context = null();
        let gl_error = CGLCreateContext(pixel_format, null(), to_unsafe_ptr(&cgl_context));
        assert gl_error == kCGLNoError;

        // Set the context.
        let gl_error = CGLSetCurrentContext(cgl_context);
        assert gl_error == kCGLNoError;

        // Lock the context.
        let gl_error = CGLLockContext(cgl_context);
        assert gl_error == kCGLNoError;

        return cgl_context;
    }
}

pub fn init_surface(+size: Size2D<int>) -> IOSurface {
    use number = core_foundation::number::CFNumber::new_number;
    use string = core_foundation::string::CFString::wrap;
    use true_value = core_foundation::boolean::CFBoolean::true_value;

    IOSurface::new_io_surface(&core_foundation::dictionary::CFDictionary::new_dictionary([
        (string(kIOSurfaceWidth),           number(size.width as i32).as_type()),
        (string(kIOSurfaceHeight),          number(size.height as i32).as_type()),
        (string(kIOSurfaceBytesPerRow),     number(size.width as i32 * 4).as_type()),
        (string(kIOSurfaceBytesPerElement), number(4i32).as_type()),
        (string(kIOSurfaceIsGlobal),        true_value().as_type())
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
pub fn bind_surface_to_texture(context: &CGLContextObj, surface: &IOSurface, +size: Size2D<int>) {
    // FIXME: There should be safe wrappers for this.
    unsafe {
        let gl_error = CGLTexImageIOSurface2D(*context,
                                              TEXTURE_RECTANGLE_ARB,
                                              RGBA as GLenum,
                                              size.width as GLsizei,
                                              size.height as GLsizei,
                                              BGRA as GLenum,
                                              UNSIGNED_INT_8_8_8_8_REV,
                                              transmute(copy surface.obj),
                                              0);
        assert gl_error == kCGLNoError;
    }
}

pub fn bind_texture_to_framebuffer(texture: GLuint) {
    gl2::bind_texture(TEXTURE_RECTANGLE_ARB, 0);
    gl2::framebuffer_texture_2d(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_RECTANGLE_ARB, texture, 0);
    assert gl2::check_framebuffer_status(FRAMEBUFFER) == FRAMEBUFFER_COMPLETE;
}

impl MacContext : ShareContext {
    static fn new(+size: Size2D<int>) -> MacContext {
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

        MacContext {
            surface: move surface,
            framebuffer: framebuffer,
            texture: texture
        }
    }

    fn flush(&self) {
        gl2::finish();
    }

    fn id() -> int {
        self.surface.get_id() as int
    }
}

