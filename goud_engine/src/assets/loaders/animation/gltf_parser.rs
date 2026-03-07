//! GLTF animation extraction.

#[cfg(feature = "native")]
use super::asset::KeyframeAnimation;
#[cfg(feature = "native")]
use super::keyframe::{AnimationChannel, EasingFunction, Keyframe};
#[cfg(feature = "native")]
use crate::assets::AssetLoadError;

/// Parses animation data from GLTF/GLB bytes.
///
/// Extracts the first animation from the GLTF document. Each channel
/// maps to an [`AnimationChannel`] with target property formatted as
/// `"node_{index}.{property}"`.
#[cfg(feature = "native")]
pub fn parse_gltf_animation(bytes: &[u8]) -> Result<KeyframeAnimation, AssetLoadError> {
    let gltf = gltf::Gltf::from_slice(bytes)
        .map_err(|e| AssetLoadError::decode_failed(format!("Failed to parse GLTF: {e}")))?;

    let buffers = load_buffers(&gltf)?;

    let gltf_anim = gltf
        .animations()
        .next()
        .ok_or_else(|| AssetLoadError::decode_failed("GLTF contains no animations"))?;

    let name = gltf_anim.name().unwrap_or("unnamed").to_string();

    let mut channels = Vec::new();
    let mut max_time: f32 = 0.0;

    for channel in gltf_anim.channels() {
        let target = channel.target();
        let node_index = target.node().index();
        let property = match target.property() {
            gltf::animation::Property::Translation => "translation",
            gltf::animation::Property::Rotation => "rotation",
            gltf::animation::Property::Scale => "scale",
            gltf::animation::Property::MorphTargetWeights => "morph_weights",
        };

        let reader = channel.reader(|buf| buffers.get(buf.index()).map(|b| b.as_slice()));

        let timestamps: Vec<f32> = match reader.read_inputs() {
            Some(inputs) => inputs.collect(),
            None => continue,
        };

        let values: Vec<f32> = match reader.read_outputs() {
            Some(outputs) => match outputs {
                gltf::animation::util::ReadOutputs::Translations(v) => {
                    v.flat_map(|t| t.into_iter()).collect()
                }
                gltf::animation::util::ReadOutputs::Rotations(v) => {
                    v.into_f32().flat_map(|r| r.into_iter()).collect()
                }
                gltf::animation::util::ReadOutputs::Scales(v) => {
                    v.flat_map(|s| s.into_iter()).collect()
                }
                gltf::animation::util::ReadOutputs::MorphTargetWeights(v) => v.into_f32().collect(),
            },
            None => continue,
        };

        // Determine component count per keyframe
        let component_count = if timestamps.is_empty() {
            1
        } else {
            values.len() / timestamps.len()
        };

        // Create a channel per component
        for comp in 0..component_count {
            let suffix = match component_count {
                3 => ["x", "y", "z"][comp],
                4 => ["x", "y", "z", "w"][comp],
                _ => "value",
            };

            let target_property = format!("node_{node_index}.{property}.{suffix}");

            let keyframes: Vec<Keyframe> = timestamps
                .iter()
                .enumerate()
                .map(|(i, &time)| {
                    let value = values
                        .get(i * component_count + comp)
                        .copied()
                        .unwrap_or(0.0);
                    if time > max_time {
                        max_time = time;
                    }
                    Keyframe {
                        time,
                        value,
                        easing: EasingFunction::Linear,
                    }
                })
                .collect();

            channels.push(AnimationChannel {
                target_property,
                keyframes,
            });
        }
    }

    Ok(KeyframeAnimation::new(name, max_time, channels))
}

/// Loads buffer data from GLTF, supporting both embedded (GLB) and data URIs.
#[cfg(feature = "native")]
fn load_buffers(gltf: &gltf::Gltf) -> Result<Vec<Vec<u8>>, AssetLoadError> {
    let mut buffers = Vec::new();

    for buffer in gltf.buffers() {
        let data = match buffer.source() {
            gltf::buffer::Source::Bin => gltf
                .blob
                .as_deref()
                .ok_or_else(|| AssetLoadError::decode_failed("GLB binary chunk missing"))?
                .to_vec(),
            gltf::buffer::Source::Uri(uri) => {
                if let Some(encoded) = uri.strip_prefix("data:application/octet-stream;base64,") {
                    decode_base64(encoded)?
                } else if let Some(encoded) =
                    uri.strip_prefix("data:application/gltf-buffer;base64,")
                {
                    decode_base64(encoded)?
                } else {
                    // External file reference -- not supported in asset loader context
                    // (we only have the primary file bytes)
                    return Err(AssetLoadError::decode_failed(format!(
                        "External buffer URI not supported in animation loader: {uri}"
                    )));
                }
            }
        };
        buffers.push(data);
    }

    Ok(buffers)
}

/// Minimal base64 decoder.
#[cfg(feature = "native")]
fn decode_base64(input: &str) -> Result<Vec<u8>, AssetLoadError> {
    let input = input.trim();
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;

    for c in input.bytes() {
        let val = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' | b'\n' | b'\r' | b' ' => continue,
            _ => {
                return Err(AssetLoadError::decode_failed(format!(
                    "Invalid base64 character: {c}"
                )))
            }
        };
        buf = (buf << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }

    Ok(output)
}
