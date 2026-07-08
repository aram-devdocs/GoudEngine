use std::sync::mpsc::{self, Receiver, TryRecvError};

use crate::libs::graphics::frame_timing;

pub(super) const TIMESTAMP_QUERY_COUNT: u32 = 6;
pub(super) const SHADOW_BEGIN_QUERY: u32 = 0;
pub(super) const SHADOW_END_QUERY: u32 = 1;
pub(super) const RENDER_BEGIN_QUERY: u32 = 2;
pub(super) const RENDER_END_QUERY: u32 = 3;
pub(super) const SUBMIT_BEGIN_QUERY: u32 = 4;
pub(super) const SUBMIT_END_QUERY: u32 = 5;
pub(super) const TIMESTAMP_BUFFER_SIZE: u64 =
    (TIMESTAMP_QUERY_COUNT as u64) * std::mem::size_of::<u64>() as u64;
pub(super) const SUBMIT_MARKER_COPY_SIZE: u64 = std::mem::size_of::<u64>() as u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct GpuTimestampFrameTimings {
    pub(super) shadow_us: u64,
    pub(super) render_us: u64,
    pub(super) submit_us: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimestampPassMode {
    RenderPassWrites,
    EncoderWrites,
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
    pass_mode: TimestampPassMode,
    latest: GpuTimestampFrameTimings,
    next_slot: usize,
}

impl GpuTimestampQueries {
    pub(super) fn requested_features(adapter_features: wgpu::Features) -> wgpu::Features {
        let required_encoder =
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;

        if !adapter_features.contains(required_encoder) {
            return wgpu::Features::empty();
        }

        if adapter_features.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES) {
            required_encoder | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES
        } else {
            required_encoder
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

        let pass_mode = if required.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES) {
            TimestampPassMode::RenderPassWrites
        } else {
            TimestampPassMode::EncoderWrites
        };

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
            pass_mode,
            latest: GpuTimestampFrameTimings::default(),
            next_slot: 0,
        })
    }

    pub(super) fn shadow_pass_writes(&self) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        (self.pass_mode == TimestampPassMode::RenderPassWrites).then_some(
            wgpu::RenderPassTimestampWrites {
                query_set: &self.query_set,
                beginning_of_pass_write_index: Some(SHADOW_BEGIN_QUERY),
                end_of_pass_write_index: Some(SHADOW_END_QUERY),
            },
        )
    }

    pub(super) fn render_pass_writes(&self) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        (self.pass_mode == TimestampPassMode::RenderPassWrites).then_some(
            wgpu::RenderPassTimestampWrites {
                query_set: &self.query_set,
                beginning_of_pass_write_index: Some(RENDER_BEGIN_QUERY),
                end_of_pass_write_index: Some(RENDER_END_QUERY),
            },
        )
    }

    pub(super) fn write_shadow_begin(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.pass_mode == TimestampPassMode::EncoderWrites {
            encoder.write_timestamp(&self.query_set, SHADOW_BEGIN_QUERY);
        }
    }

    pub(super) fn write_shadow_end(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.pass_mode == TimestampPassMode::EncoderWrites {
            encoder.write_timestamp(&self.query_set, SHADOW_END_QUERY);
        }
    }

    pub(super) fn write_empty_shadow_phase(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.write_timestamp(&self.query_set, SHADOW_BEGIN_QUERY);
        encoder.write_timestamp(&self.query_set, SHADOW_END_QUERY);
    }

    pub(super) fn write_render_begin(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.pass_mode == TimestampPassMode::EncoderWrites {
            encoder.write_timestamp(&self.query_set, RENDER_BEGIN_QUERY);
        }
    }

    pub(super) fn write_render_end(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.pass_mode == TimestampPassMode::EncoderWrites {
            encoder.write_timestamp(&self.query_set, RENDER_END_QUERY);
        }
    }

    pub(super) fn resolve_into_readback(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<usize> {
        let slot_index = self.next_available_slot()?;
        let slot = &self.readback_slots[slot_index];

        // wgpu timestamps can only measure GPU command-stream work, not the CPU-side
        // queue.submit() call itself. This marker brackets the encoded submission tail
        // before the query resolve/copy that makes the timestamps readable next frame.
        encoder.write_timestamp(&self.query_set, SUBMIT_BEGIN_QUERY);
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffer,
            0,
            &slot.buffer,
            0,
            SUBMIT_MARKER_COPY_SIZE,
        );
        encoder.write_timestamp(&self.query_set, SUBMIT_END_QUERY);
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
