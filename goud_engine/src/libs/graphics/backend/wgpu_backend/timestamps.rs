use std::sync::mpsc::{self, Receiver, TryRecvError};

use crate::libs::graphics::frame_timing;

const TIMESTAMP_QUERY_COUNT: u32 = 6;
const SHADOW_BEGIN_QUERY: u32 = 0;
const SHADOW_END_QUERY: u32 = 1;
const RENDER_BEGIN_QUERY: u32 = 2;
const RENDER_END_QUERY: u32 = 3;
const SUBMIT_BEGIN_QUERY: u32 = 4;
const SUBMIT_END_QUERY: u32 = 5;
const TIMESTAMP_BUFFER_SIZE: u64 =
    (TIMESTAMP_QUERY_COUNT as u64) * std::mem::size_of::<u64>() as u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct GpuTimestampFrameTimings {
    pub(super) shadow_us: u64,
    pub(super) render_us: u64,
    pub(super) submit_us: u64,
}

#[derive(Debug)]
struct PendingTimestampReadback {
    receiver: Receiver<Result<(), wgpu::BufferAsyncError>>,
}

#[derive(Debug)]
struct TimestampReadbackSlot {
    buffer: wgpu::Buffer,
    pending: Option<PendingTimestampReadback>,
}

#[derive(Debug)]
pub(super) struct GpuTimestampQueries {
    query_set: wgpu::QuerySet,
    resolve_buffer: wgpu::Buffer,
    readback_slots: [TimestampReadbackSlot; 2],
    timestamp_period_ns: f32,
    latest: GpuTimestampFrameTimings,
    next_slot: usize,
}

/// Result of the headless timestamp-query probe used by the ENG2 spec test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuTimestampProbeReport {
    /// Whether the selected adapter exposes the timestamp-query feature set.
    pub supported: bool,
    /// Raw timestamp slots resolved from the GPU query set.
    pub raw_queries: [u64; TIMESTAMP_QUERY_COUNT as usize],
}

impl GpuTimestampQueries {
    pub(super) fn requested_features(adapter_features: wgpu::Features) -> wgpu::Features {
        let required = wgpu::Features::TIMESTAMP_QUERY
            | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS
            | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES;

        if adapter_features.contains(required) {
            required
        } else {
            wgpu::Features::empty()
        }
    }

    pub(super) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        enabled_features: wgpu::Features,
    ) -> Option<Self> {
        let required = Self::requested_features(enabled_features);
        if required.is_empty() {
            return None;
        }

        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("goud-gpu-timestamps"),
            ty: wgpu::QueryType::Timestamp,
            count: TIMESTAMP_QUERY_COUNT,
        });
        let resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("goud-gpu-timestamp-resolve"),
            size: TIMESTAMP_BUFFER_SIZE,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback_slots = std::array::from_fn(|index| TimestampReadbackSlot {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(match index {
                    0 => "goud-gpu-timestamp-readback-a",
                    _ => "goud-gpu-timestamp-readback-b",
                }),
                size: TIMESTAMP_BUFFER_SIZE,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            pending: None,
        });

        Some(Self {
            query_set,
            resolve_buffer,
            readback_slots,
            timestamp_period_ns: queue.get_timestamp_period(),
            latest: GpuTimestampFrameTimings::default(),
            next_slot: 0,
        })
    }

    pub(super) fn shadow_pass_writes(&self) -> wgpu::RenderPassTimestampWrites<'_> {
        wgpu::RenderPassTimestampWrites {
            query_set: &self.query_set,
            beginning_of_pass_write_index: Some(SHADOW_BEGIN_QUERY),
            end_of_pass_write_index: Some(SHADOW_END_QUERY),
        }
    }

    pub(super) fn render_pass_writes(&self) -> wgpu::RenderPassTimestampWrites<'_> {
        wgpu::RenderPassTimestampWrites {
            query_set: &self.query_set,
            beginning_of_pass_write_index: Some(RENDER_BEGIN_QUERY),
            end_of_pass_write_index: Some(RENDER_END_QUERY),
        }
    }

    pub(super) fn resolve_into_readback(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<usize> {
        let slot_index = self.next_available_slot()?;
        let slot = &self.readback_slots[slot_index];

        // wgpu timestamps can only measure work inside the GPU command stream, not the
        // CPU-side queue.submit() call itself. We therefore treat gpu_submit as the
        // submission tail: query resolution plus the copy into the MAP_READ buffer.
        encoder.write_timestamp(&self.query_set, SUBMIT_BEGIN_QUERY);
        encoder.resolve_query_set(
            &self.query_set,
            0..TIMESTAMP_QUERY_COUNT,
            &self.resolve_buffer,
            0,
        );
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffer,
            0,
            &slot.buffer,
            0,
            TIMESTAMP_BUFFER_SIZE,
        );
        encoder.write_timestamp(&self.query_set, SUBMIT_END_QUERY);

        self.next_slot = (slot_index + 1) % self.readback_slots.len();
        Some(slot_index)
    }

    pub(super) fn begin_readback(&mut self, slot_index: usize) {
        let Some(slot) = self.readback_slots.get_mut(slot_index) else {
            return;
        };
        if slot.pending.is_some() {
            return;
        }

        let buffer_slice = slot.buffer.slice(..);
        let (tx, rx) = mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        slot.pending = Some(PendingTimestampReadback { receiver: rx });
    }

    pub(super) fn poll_ready_results(&mut self, device: &wgpu::Device) {
        let has_pending = self
            .readback_slots
            .iter()
            .any(|slot| slot.pending.is_some());
        if !has_pending {
            return;
        }

        let _ = device.poll(wgpu::PollType::Poll);

        for slot in &mut self.readback_slots {
            let Some(pending) = slot.pending.take() else {
                continue;
            };

            match pending.receiver.try_recv() {
                Ok(Ok(())) => {
                    if let Some(timings) =
                        read_timestamp_timings(&slot.buffer, self.timestamp_period_ns)
                    {
                        self.latest = timings;
                    }
                    slot.buffer.unmap();
                }
                Ok(Err(_)) | Err(TryRecvError::Disconnected) => {
                    slot.buffer.unmap();
                }
                Err(TryRecvError::Empty) => {
                    slot.pending = Some(pending);
                }
            }
        }
    }

    pub(super) fn record_latest_timings(&self) {
        frame_timing::record_phase("gpu_shadow", self.latest.shadow_us);
        frame_timing::record_phase("gpu_render", self.latest.render_us);
        frame_timing::record_phase("gpu_submit", self.latest.submit_us);
    }

    fn next_available_slot(&self) -> Option<usize> {
        (0..self.readback_slots.len())
            .map(|offset| (self.next_slot + offset) % self.readback_slots.len())
            .find(|&index| self.readback_slots[index].pending.is_none())
    }
}

fn read_timestamp_timings(
    buffer: &wgpu::Buffer,
    timestamp_period_ns: f32,
) -> Option<GpuTimestampFrameTimings> {
    let mapped = buffer.slice(..).get_mapped_range();
    let raw = bytemuck::cast_slice::<u8, u64>(&mapped);
    if raw.len() < TIMESTAMP_QUERY_COUNT as usize {
        return None;
    }

    let timings = GpuTimestampFrameTimings {
        shadow_us: raw_timestamp_delta_us(
            raw[SHADOW_BEGIN_QUERY as usize],
            raw[SHADOW_END_QUERY as usize],
            timestamp_period_ns,
        ),
        render_us: raw_timestamp_delta_us(
            raw[RENDER_BEGIN_QUERY as usize],
            raw[RENDER_END_QUERY as usize],
            timestamp_period_ns,
        ),
        submit_us: raw_timestamp_delta_us(
            raw[SUBMIT_BEGIN_QUERY as usize],
            raw[SUBMIT_END_QUERY as usize],
            timestamp_period_ns,
        ),
    };
    drop(mapped);
    Some(timings)
}

fn raw_timestamp_delta_us(start: u64, end: u64, timestamp_period_ns: f32) -> u64 {
    if end <= start {
        return 0;
    }

    let delta_ticks = end - start;
    ((delta_ticks as f64) * (timestamp_period_ns as f64) / 1_000.0) as u64
}

/// Runs a headless wgpu timestamp-query pass pair and resolves the raw query slots.
pub fn probe_gpu_timestamp_queries() -> Result<GpuTimestampProbeReport, String> {
    pollster::block_on(probe_gpu_timestamp_queries_async())
}

async fn probe_gpu_timestamp_queries_async() -> Result<GpuTimestampProbeReport, String> {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: super::force_fallback_adapter(),
        })
        .await
        .map_err(|e| format!("No suitable headless adapter: {e}"))?;

    let features = GpuTimestampQueries::requested_features(adapter.features());
    if features.is_empty() {
        return Ok(GpuTimestampProbeReport {
            supported: false,
            raw_queries: [0; TIMESTAMP_QUERY_COUNT as usize],
        });
    }

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("goud-gpu-timestamp-probe"),
            required_features: features,
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        })
        .await
        .map_err(|e| format!("Failed to create probe device: {e}"))?;

    let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
        label: Some("goud-gpu-timestamp-probe"),
        ty: wgpu::QueryType::Timestamp,
        count: TIMESTAMP_QUERY_COUNT,
    });
    let resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("goud-gpu-timestamp-probe-resolve"),
        size: TIMESTAMP_BUFFER_SIZE,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("goud-gpu-timestamp-probe-readback"),
        size: TIMESTAMP_BUFFER_SIZE,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("goud-gpu-timestamp-probe-shadow"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let color_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("goud-gpu-timestamp-probe-color"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("goud-gpu-timestamp-probe-depth"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("goud-gpu-timestamp-probe"),
    });
    {
        let _shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("goud-gpu-timestamp-probe-shadow"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &shadow_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: Some(wgpu::RenderPassTimestampWrites {
                query_set: &query_set,
                beginning_of_pass_write_index: Some(SHADOW_BEGIN_QUERY),
                end_of_pass_write_index: Some(SHADOW_END_QUERY),
            }),
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("goud-gpu-timestamp-probe-render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: Some(wgpu::RenderPassTimestampWrites {
                query_set: &query_set,
                beginning_of_pass_write_index: Some(RENDER_BEGIN_QUERY),
                end_of_pass_write_index: Some(RENDER_END_QUERY),
            }),
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }

    encoder.write_timestamp(&query_set, SUBMIT_BEGIN_QUERY);
    encoder.resolve_query_set(&query_set, 0..TIMESTAMP_QUERY_COUNT, &resolve_buffer, 0);
    encoder.copy_buffer_to_buffer(
        &resolve_buffer,
        0,
        &readback_buffer,
        0,
        TIMESTAMP_BUFFER_SIZE,
    );
    encoder.write_timestamp(&query_set, SUBMIT_END_QUERY);

    queue.submit(std::iter::once(encoder.finish()));

    let slice = readback_buffer.slice(..);
    let (tx, rx) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|e| format!("Probe poll failed: {e}"))?;
    rx.recv()
        .map_err(|e| format!("Probe map receive failed: {e}"))?
        .map_err(|e| format!("Probe map failed: {e}"))?;

    let mapped = slice.get_mapped_range();
    let raw = bytemuck::cast_slice::<u8, u64>(&mapped);
    let mut raw_queries = [0; TIMESTAMP_QUERY_COUNT as usize];
    raw_queries.copy_from_slice(&raw[..TIMESTAMP_QUERY_COUNT as usize]);
    drop(mapped);
    readback_buffer.unmap();

    Ok(GpuTimestampProbeReport {
        supported: true,
        raw_queries,
    })
}
