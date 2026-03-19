use super::super::asset::MeshMaterial;
use crate::assets::loaders::gltf_utils::decode_data_uri;
use crate::assets::{AssetLoadError, AssetPath, LoadContext};
use cgmath::{InnerSpace, Matrix, Matrix4, SquareMatrix, Vector3, Vector4};
use std::path::Path;

pub(super) fn collect_buffer_data(
    document: &gltf::Document,
    blob: &mut Option<Vec<u8>>,
    context: &mut LoadContext,
) -> Result<Vec<Vec<u8>>, AssetLoadError> {
    let mut buffers = Vec::new();
    for buffer in document.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                let data = blob.take().ok_or_else(|| {
                    AssetLoadError::decode_failed("GLB missing binary blob for buffer")
                })?;
                buffers.push(data);
            }
            gltf::buffer::Source::Uri(uri) => {
                if uri.starts_with("data:") {
                    buffers.push(decode_data_uri(uri)?);
                } else {
                    let path = resolve_relative_asset_path(context.path(), uri);
                    context.add_dependency(path.clone());
                    buffers.push(context.read_asset_bytes(&path)?);
                }
            }
        }
    }
    Ok(buffers)
}

pub(super) fn collect_image_assets(
    document: &gltf::Document,
    buffers: &[Vec<u8>],
    context: &mut LoadContext,
) -> Result<Vec<Option<String>>, AssetLoadError> {
    let mut image_paths = vec![None; document.images().len()];
    for image in document.images() {
        let path = match image.source() {
            gltf::image::Source::Uri { uri, mime_type } => {
                if uri.starts_with("data:") {
                    let extension =
                        extension_for_image_source(mime_type, Some(uri)).ok_or_else(|| {
                            AssetLoadError::decode_failed(format!(
                                "Unsupported embedded GLTF image source for image {}",
                                image.index()
                            ))
                        })?;
                    let path = embedded_image_path(context.path(), image.index(), extension);
                    context.add_embedded_asset(path.clone(), decode_data_uri(uri)?);
                    path
                } else {
                    let path = resolve_relative_asset_path(context.path(), uri);
                    context.add_dependency(path.clone());
                    path
                }
            }
            gltf::image::Source::View { view, mime_type } => {
                let buffer = buffers.get(view.buffer().index()).ok_or_else(|| {
                    AssetLoadError::decode_failed(format!(
                        "GLTF image buffer {} missing",
                        view.buffer().index()
                    ))
                })?;
                let start = view.offset();
                let end = start + view.length();
                let bytes = buffer.get(start..end).ok_or_else(|| {
                    AssetLoadError::decode_failed(format!(
                        "GLTF image view {} out of bounds",
                        image.index()
                    ))
                })?;
                let extension =
                    extension_for_image_source(Some(mime_type), None).ok_or_else(|| {
                        AssetLoadError::decode_failed(format!(
                            "Unsupported GLTF image MIME type '{mime_type}'"
                        ))
                    })?;
                let path = embedded_image_path(context.path(), image.index(), extension);
                context.add_embedded_asset(path.clone(), bytes.to_vec());
                path
            }
        };
        image_paths[image.index()] = Some(path);
    }
    Ok(image_paths)
}

pub(super) fn extract_material(
    material: gltf::Material,
    image_paths: &[Option<String>],
) -> Option<MeshMaterial> {
    let pbr = material.pbr_metallic_roughness();
    let base_color_texture = pbr
        .base_color_texture()
        .and_then(|info| texture_asset_path(info.texture(), image_paths));
    let normal_texture = material
        .normal_texture()
        .and_then(|info| texture_asset_path(info.texture(), image_paths));
    let metallic_roughness_texture = pbr
        .metallic_roughness_texture()
        .and_then(|info| texture_asset_path(info.texture(), image_paths));
    let emissive_texture = material
        .emissive_texture()
        .and_then(|info| texture_asset_path(info.texture(), image_paths));
    let alpha_cutoff = match material.alpha_mode() {
        gltf::material::AlphaMode::Mask => Some(material.alpha_cutoff().unwrap_or(0.5)),
        _ => None,
    };
    let imported = material.index().is_some()
        || base_color_texture.is_some()
        || normal_texture.is_some()
        || metallic_roughness_texture.is_some()
        || emissive_texture.is_some()
        || material.double_sided()
        || alpha_cutoff.is_some()
        || pbr.base_color_factor() != [1.0, 1.0, 1.0, 1.0]
        || material.emissive_factor() != [0.0, 0.0, 0.0]
        || (pbr.metallic_factor() - 1.0).abs() > f32::EPSILON
        || (pbr.roughness_factor() - 1.0).abs() > f32::EPSILON;
    imported.then(|| MeshMaterial {
        name: material.name().map(ToOwned::to_owned),
        base_color_factor: pbr.base_color_factor(),
        base_color_texture_path: base_color_texture,
        normal_texture_path: normal_texture,
        metallic_roughness_texture_path: metallic_roughness_texture,
        emissive_texture_path: emissive_texture,
        emissive_factor: material.emissive_factor(),
        metallic_factor: pbr.metallic_factor(),
        roughness_factor: pbr.roughness_factor(),
        alpha_cutoff,
        double_sided: material.double_sided(),
    })
}

fn texture_asset_path(texture: gltf::Texture, image_paths: &[Option<String>]) -> Option<String> {
    image_paths
        .get(texture.source().index())
        .and_then(|path| path.clone())
}

pub(super) fn primitive_name(
    node: Option<&gltf::Node>,
    mesh: &gltf::Mesh,
    primitive: &gltf::Primitive,
) -> String {
    let node_name = node
        .and_then(|node| node.name().map(ToOwned::to_owned))
        .unwrap_or_else(|| format!("node_{}", node.map_or(0, |node| node.index())));
    let mesh_name = mesh
        .name()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("mesh_{}", mesh.index()));
    format!("{node_name}.{mesh_name}.primitive_{}", primitive.index())
}

fn resolve_relative_asset_path(base: &AssetPath<'_>, uri: &str) -> String {
    let uri = uri.split('#').next().unwrap_or(uri);
    let path = if let Some(directory) = base.directory() {
        Path::new(directory).join(uri)
    } else {
        Path::new(uri).to_path_buf()
    };
    AssetPath::from_path(&path).into_owned().to_string()
}

fn embedded_image_path(base: &AssetPath<'_>, image_index: usize, extension: &str) -> String {
    let stem = base.stem().unwrap_or("mesh");
    let file_name = format!("{stem}__embedded_image_{image_index}.{extension}");
    if let Some(directory) = base.directory() {
        AssetPath::new(directory).join(&file_name).to_string()
    } else {
        file_name
    }
}

fn extension_for_image_source(mime_type: Option<&str>, uri: Option<&str>) -> Option<&'static str> {
    mime_type
        .and_then(extension_for_mime_type)
        .or_else(|| uri.and_then(extension_for_uri))
}

fn extension_for_mime_type(mime_type: &str) -> Option<&'static str> {
    match mime_type {
        "image/png" => Some("png"),
        "image/jpeg" | "image/jpg" => Some("jpg"),
        "image/webp" => Some("webp"),
        "image/bmp" => Some("bmp"),
        "image/gif" => Some("gif"),
        _ => None,
    }
}

fn extension_for_uri(uri: &str) -> Option<&'static str> {
    if let Some(data_uri_extension) = extension_for_data_uri(uri) {
        return Some(data_uri_extension);
    }
    let path = uri
        .split('#')
        .next()
        .unwrap_or(uri)
        .split('?')
        .next()
        .unwrap_or(uri);
    match Path::new(path).extension().and_then(|ext| ext.to_str()) {
        Some("png") => Some("png"),
        Some("jpg") | Some("jpeg") => Some("jpg"),
        Some("webp") => Some("webp"),
        Some("bmp") => Some("bmp"),
        Some("gif") => Some("gif"),
        _ => None,
    }
}

fn extension_for_data_uri(uri: &str) -> Option<&'static str> {
    let prefix = "data:";
    if !uri.starts_with(prefix) {
        return None;
    }
    let mime_end = uri.find(';').or_else(|| uri.find(','))?;
    extension_for_mime_type(&uri[prefix.len()..mime_end])
}

pub(super) fn node_transform_matrix(matrix: [[f32; 4]; 4]) -> Matrix4<f32> {
    Matrix4::new(
        matrix[0][0],
        matrix[0][1],
        matrix[0][2],
        matrix[0][3],
        matrix[1][0],
        matrix[1][1],
        matrix[1][2],
        matrix[1][3],
        matrix[2][0],
        matrix[2][1],
        matrix[2][2],
        matrix[2][3],
        matrix[3][0],
        matrix[3][1],
        matrix[3][2],
        matrix[3][3],
    )
}

pub(super) fn transform_position(transform: Matrix4<f32>, position: [f32; 3]) -> [f32; 3] {
    let transformed = transform * Vector4::new(position[0], position[1], position[2], 1.0);
    [transformed.x, transformed.y, transformed.z]
}

pub(super) fn transform_normal(transform: Matrix4<f32>, normal: [f32; 3]) -> [f32; 3] {
    let basis = transform
        .invert()
        .unwrap_or_else(Matrix4::identity)
        .transpose();
    let transformed = basis * Vector4::new(normal[0], normal[1], normal[2], 0.0);
    let normalized = Vector3::new(transformed.x, transformed.y, transformed.z);
    if normalized.magnitude2() > 0.0 {
        let normalized = normalized.normalize();
        [normalized.x, normalized.y, normalized.z]
    } else {
        [0.0, 0.0, 1.0]
    }
}
