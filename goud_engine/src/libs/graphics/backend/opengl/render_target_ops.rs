//! OpenGL render-target creation, binding, and destruction.

use super::{backend::OpenGLBackend, gl_check_debug, texture_ops, RenderTargetMetadata};
use crate::libs::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::render_backend::RenderTargetOps;
use crate::libs::graphics::backend::types::{
    RenderTargetDesc, RenderTargetHandle, TextureFilter, TextureHandle, TextureWrap,
};

fn framebuffer_status_to_string(status: u32) -> &'static str {
    match status {
        gl::FRAMEBUFFER_COMPLETE => "GL_FRAMEBUFFER_COMPLETE",
        gl::FRAMEBUFFER_UNDEFINED => "GL_FRAMEBUFFER_UNDEFINED",
        gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT",
        gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
            "GL_FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT"
        }
        gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => "GL_FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER",
        gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => "GL_FRAMEBUFFER_INCOMPLETE_READ_BUFFER",
        gl::FRAMEBUFFER_UNSUPPORTED => "GL_FRAMEBUFFER_UNSUPPORTED",
        gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => "GL_FRAMEBUFFER_INCOMPLETE_MULTISAMPLE",
        gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS => "GL_FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS",
        _ => "GL_FRAMEBUFFER_STATUS_UNKNOWN",
    }
}

impl RenderTargetOps for OpenGLBackend {
    fn create_render_target(&mut self, desc: &RenderTargetDesc) -> GoudResult<RenderTargetHandle> {
        if desc.width == 0 || desc.height == 0 {
            return Err(GoudError::RenderTargetFailed(
                "Render target dimensions must be greater than 0".to_string(),
            ));
        }

        let color_texture = texture_ops::create_texture(
            self,
            desc.width,
            desc.height,
            desc.format,
            TextureFilter::Linear,
            TextureWrap::ClampToEdge,
            &[],
        )?;

        let texture_gl_id = self
            .textures
            .get(&color_texture)
            .map(|meta| meta.gl_id)
            .ok_or_else(|| {
                GoudError::RenderTargetFailed(
                    "Render target texture metadata is missing".to_string(),
                )
            })?;

        let mut framebuffer_id = 0u32;
        let mut depth_renderbuffer = None;
        let mut previous_framebuffer = 0i32;

        // SAFETY: A valid OpenGL context is active for the lifetime of the backend.
        // The framebuffer and optional renderbuffer IDs are stack outputs owned by this function
        // until inserted into backend state.
        unsafe {
            gl::GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut previous_framebuffer);
            gl::GenFramebuffers(1, &mut framebuffer_id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer_id);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture_gl_id,
                0,
            );

            if desc.has_depth {
                let mut renderbuffer_id = 0u32;
                gl::GenRenderbuffers(1, &mut renderbuffer_id);
                gl::BindRenderbuffer(gl::RENDERBUFFER, renderbuffer_id);
                gl::RenderbufferStorage(
                    gl::RENDERBUFFER,
                    gl::DEPTH_COMPONENT24,
                    desc.width as i32,
                    desc.height as i32,
                );
                gl::FramebufferRenderbuffer(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_ATTACHMENT,
                    gl::RENDERBUFFER,
                    renderbuffer_id,
                );
                gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
                depth_renderbuffer = Some(renderbuffer_id);
            }

            let draw_buffers = [gl::COLOR_ATTACHMENT0];
            gl::DrawBuffers(draw_buffers.len() as i32, draw_buffers.as_ptr());

            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            gl::BindFramebuffer(gl::FRAMEBUFFER, previous_framebuffer as u32);

            if status != gl::FRAMEBUFFER_COMPLETE {
                gl::DeleteFramebuffers(1, &framebuffer_id);
                if let Some(renderbuffer_id) = depth_renderbuffer {
                    gl::DeleteRenderbuffers(1, &renderbuffer_id);
                }
                let _ = texture_ops::destroy_texture(self, color_texture);
                return Err(GoudError::RenderTargetFailed(
                    framebuffer_status_to_string(status).to_string(),
                ));
            }
        }
        gl_check_debug!("create_render_target");

        let handle = self.render_target_allocator.allocate();
        self.render_targets.insert(
            handle,
            RenderTargetMetadata {
                framebuffer_id,
                color_texture,
                depth_renderbuffer,
                width: desc.width,
                height: desc.height,
            },
        );

        Ok(handle)
    }

    fn destroy_render_target(&mut self, handle: RenderTargetHandle) -> bool {
        let Some(metadata) = self.render_targets.remove(&handle) else {
            return false;
        };

        if self.active_render_target == Some(handle) {
            let _ = self.bind_render_target(None);
        }

        // SAFETY: IDs were created by this backend and are no longer referenced after removal.
        unsafe {
            gl::DeleteFramebuffers(1, &metadata.framebuffer_id);
            if let Some(depth_renderbuffer) = metadata.depth_renderbuffer {
                gl::DeleteRenderbuffers(1, &depth_renderbuffer);
            }
        }
        gl_check_debug!("destroy_render_target");

        let _ = texture_ops::destroy_texture(self, metadata.color_texture);
        self.render_target_allocator.deallocate(handle)
    }

    fn is_render_target_valid(&self, handle: RenderTargetHandle) -> bool {
        self.render_target_allocator.is_alive(handle) && self.render_targets.contains_key(&handle)
    }

    fn bind_render_target(&mut self, handle: Option<RenderTargetHandle>) -> GoudResult<()> {
        match handle {
            Some(handle) => {
                let metadata = self
                    .render_targets
                    .get(&handle)
                    .ok_or_else(|| GoudError::InvalidHandle)?;
                // SAFETY: framebuffer_id is owned by this backend and alive while the metadata is present.
                unsafe {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, metadata.framebuffer_id);
                    gl::Viewport(0, 0, metadata.width as i32, metadata.height as i32);
                }
                self.active_render_target = Some(handle);
            }
            None => {
                let (x, y, width, height) = self.default_viewport;
                // SAFETY: Binding framebuffer 0 restores the default framebuffer for the current context.
                unsafe {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                    gl::Viewport(x, y, width as i32, height as i32);
                }
                self.active_render_target = None;
            }
        }
        gl_check_debug!("bind_render_target");
        Ok(())
    }

    fn render_target_texture(&self, handle: RenderTargetHandle) -> Option<TextureHandle> {
        self.render_targets
            .get(&handle)
            .map(|meta| meta.color_texture)
    }
}
