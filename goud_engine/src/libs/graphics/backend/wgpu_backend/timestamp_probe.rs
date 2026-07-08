use std::{
    sync::mpsc::{self, TryRecvError},
    time::{Duration, Instant},
};

use super::{
    force_fallback_adapter,
    timestamps::{
        GpuTimestampQueries, RENDER_BEGIN_QUERY, RENDER_END_QUERY, SHADOW_BEGIN_QUERY,
        SHADOW_END_QUERY, SUBMIT_BEGIN_QUERY, SUBMIT_END_QUERY, SUBMIT_MARKER_COPY_SIZE,
        TIMESTAMP_BUFFER_SIZE, TIMESTAMP_QUERY_COUNT,
    },
};

/// Result of the headless timestamp-query probe used by the ENG2 spec test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuTimestampProbeReport {
    /// Whether the selected adapter exposes the timestamp-query feature set.
    pub supported: bool,
    /// Raw timestamp slots resolved from the GPU query set.
    pub raw_queries: [u64; TIMESTAMP_QUERY_COUNT as usize],
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
            force_fallback_adapter: force_fallback_adapter(),
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

    let use_render_pass_writes = features.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES);
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
    let shadow_texture = create_probe_texture(
        &device,
        "goud-gpu-timestamp-probe-shadow",
        wgpu::TextureFormat::Depth32Float,
    );
    let color_texture = create_probe_texture(
        &device,
        "goud-gpu-timestamp-probe-color",
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let depth_texture = create_probe_texture(
        &device,
        "goud-gpu-timestamp-probe-depth",
        wgpu::TextureFormat::Depth32Float,
    );

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("goud-gpu-timestamp-probe"),
    });
    record_probe_passes(
        &mut encoder,
        &query_set,
        use_render_pass_writes,
        &shadow_texture.create_view(&wgpu::TextureViewDescriptor::default()),
        &color_texture.create_view(&wgpu::TextureViewDescriptor::default()),
        &depth_texture.create_view(&wgpu::TextureViewDescriptor::default()),
    );
    record_probe_resolve(&mut encoder, &query_set, &resolve_buffer, &readback_buffer);
    queue.submit(std::iter::once(encoder.finish()));

    read_probe_queries(&device, &readback_buffer)
}

fn create_probe_texture(
    device: &wgpu::Device,
    label: &'static str,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

fn record_probe_passes(
    encoder: &mut wgpu::CommandEncoder,
    query_set: &wgpu::QuerySet,
    use_render_pass_writes: bool,
    shadow_view: &wgpu::TextureView,
    color_view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
) {
    if !use_render_pass_writes {
        encoder.write_timestamp(query_set, SHADOW_BEGIN_QUERY);
    }
    {
        let _shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("goud-gpu-timestamp-probe-shadow"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: shadow_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: use_render_pass_writes.then_some(wgpu::RenderPassTimestampWrites {
                query_set,
                beginning_of_pass_write_index: Some(SHADOW_BEGIN_QUERY),
                end_of_pass_write_index: Some(SHADOW_END_QUERY),
            }),
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    if !use_render_pass_writes {
        encoder.write_timestamp(query_set, SHADOW_END_QUERY);
        encoder.write_timestamp(query_set, RENDER_BEGIN_QUERY);
    }
    {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("goud-gpu-timestamp-probe-render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: use_render_pass_writes.then_some(wgpu::RenderPassTimestampWrites {
                query_set,
                beginning_of_pass_write_index: Some(RENDER_BEGIN_QUERY),
                end_of_pass_write_index: Some(RENDER_END_QUERY),
            }),
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    if !use_render_pass_writes {
        encoder.write_timestamp(query_set, RENDER_END_QUERY);
    }
}

fn record_probe_resolve(
    encoder: &mut wgpu::CommandEncoder,
    query_set: &wgpu::QuerySet,
    resolve_buffer: &wgpu::Buffer,
    readback_buffer: &wgpu::Buffer,
) {
    encoder.write_timestamp(query_set, SUBMIT_BEGIN_QUERY);
    encoder.copy_buffer_to_buffer(
        resolve_buffer,
        0,
        readback_buffer,
        0,
        SUBMIT_MARKER_COPY_SIZE,
    );
    encoder.write_timestamp(query_set, SUBMIT_END_QUERY);
    encoder.resolve_query_set(query_set, 0..TIMESTAMP_QUERY_COUNT, resolve_buffer, 0);
    encoder.copy_buffer_to_buffer(resolve_buffer, 0, readback_buffer, 0, TIMESTAMP_BUFFER_SIZE);
}

fn read_probe_queries(
    device: &wgpu::Device,
    readback_buffer: &wgpu::Buffer,
) -> Result<GpuTimestampProbeReport, String> {
    let slice = readback_buffer.slice(..);
    let (tx, rx) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        device
            .poll(wgpu::PollType::Poll)
            .map_err(|e| format!("Probe poll failed: {e}"))?;
        match rx.try_recv() {
            Ok(result) => {
                result.map_err(|e| format!("Probe map failed: {e}"))?;
                break;
            }
            Err(TryRecvError::Disconnected) => {
                return Err("Probe map callback disconnected".to_string());
            }
            Err(TryRecvError::Empty) if Instant::now() >= deadline => {
                return Err("Timed out waiting for timestamp probe readback".to_string());
            }
            Err(TryRecvError::Empty) => {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }

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
