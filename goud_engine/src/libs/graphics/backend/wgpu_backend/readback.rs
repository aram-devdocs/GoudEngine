use super::{BackendInfo, ClearOps, RenderBackend, WgpuBackend};
use crate::libs::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::render_backend::RenderTargetOps;

pub(super) struct PendingFrameReadback {
    pub(super) buffer: wgpu::Buffer,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) padded_bytes_per_row: u32,
    pub(super) unpadded_bytes_per_row: u32,
}

impl RenderBackend for WgpuBackend {
    fn info(&self) -> &BackendInfo {
        &self.info
    }

    fn read_default_framebuffer_rgba8(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, String> {
        if !self.surface_supports_copy_src {
            return Err(
                "default framebuffer readback is not supported by this wgpu surface".into(),
            );
        }

        match self.last_frame_readback.as_ref() {
            Some((cached_width, cached_height, rgba8))
                if *cached_width == width && *cached_height == height =>
            {
                Ok(rgba8.clone())
            }
            Some((cached_width, cached_height, _)) => Err(format!(
                "framebuffer readback size mismatch: requested {}x{}, last frame was {}x{}",
                width, height, cached_width, cached_height
            )),
            None => Err("no completed wgpu frame is available for readback".to_string()),
        }
    }
}

impl RenderTargetOps for WgpuBackend {}

impl WgpuBackend {
    pub(super) fn prepare_frame_readback(&self) -> PendingFrameReadback {
        let width = self.surface_config.width.max(1);
        let height = self.surface_config.height.max(1);
        let unpadded_bytes_per_row = width * 4;
        let padded_bytes_per_row = unpadded_bytes_per_row
            .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer_size = padded_bytes_per_row as u64 * height as u64;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wgpu-frame-readback"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        PendingFrameReadback {
            buffer,
            width,
            height,
            padded_bytes_per_row,
            unpadded_bytes_per_row,
        }
    }

    pub(super) fn finish_frame_readback(
        &mut self,
        readback: PendingFrameReadback,
    ) -> GoudResult<()> {
        let buffer_slice = readback.buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| GoudError::InternalError(format!("wgpu readback poll failed: {e}")))?;
        rx.recv()
            .map_err(|e| GoudError::InternalError(format!("wgpu readback recv failed: {e}")))?
            .map_err(|e| GoudError::InternalError(format!("wgpu readback map failed: {e}")))?;

        let mapped = buffer_slice.get_mapped_range();
        let mut rgba8 = vec![0u8; (readback.width * readback.height * 4) as usize];
        for row in 0..readback.height as usize {
            let src_offset = row * readback.padded_bytes_per_row as usize;
            let dst_offset = row * readback.unpadded_bytes_per_row as usize;
            let src_end = src_offset + readback.unpadded_bytes_per_row as usize;
            let dst_end = dst_offset + readback.unpadded_bytes_per_row as usize;
            rgba8[dst_offset..dst_end].copy_from_slice(&mapped[src_offset..src_end]);
        }
        drop(mapped);
        readback.buffer.unmap();
        self.last_frame_readback = Some((readback.width, readback.height, rgba8));
        Ok(())
    }
}

impl ClearOps for WgpuBackend {
    fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear_color = wgpu::Color {
            r: r as f64,
            g: g as f64,
            b: b as f64,
            a: a as f64,
        };
    }

    fn clear_color(&mut self) {
        self.needs_clear = true;
    }

    fn clear_depth(&mut self) {
        self.needs_clear = true;
    }
}
