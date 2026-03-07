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
    use crate::assets::loaders::gltf_utils::decode_data_uri;

    let mut buffers = Vec::new();

    for buffer in gltf.buffers() {
        let data = match buffer.source() {
            gltf::buffer::Source::Bin => gltf
                .blob
                .as_deref()
                .ok_or_else(|| AssetLoadError::decode_failed("GLB binary chunk missing"))?
                .to_vec(),
            gltf::buffer::Source::Uri(uri) => {
                if uri.starts_with("data:") {
                    decode_data_uri(uri)?
                } else {
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
