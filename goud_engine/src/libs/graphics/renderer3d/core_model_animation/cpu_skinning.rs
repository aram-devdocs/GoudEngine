//! CPU skinning: apply skeletal deformation on the CPU.

/// Minimum individual bone weight to contribute to skinning.
///
/// Bones with weight below this threshold are skipped, saving unnecessary
/// matrix multiplications for near-zero influences.
const WEIGHT_EPSILON: f32 = 0.001;

/// Minimum accumulated bone weight for a vertex to be skinned.
///
/// Vertices with total weight below this threshold retain their original
/// bind-pose position and normal instead of collapsing to the origin.
const SKIN_WEIGHT_EPSILON: f32 = 1e-6;

/// Apply skeletal deformation to a bind-pose sub-mesh on the CPU.
///
/// The bind-pose buffer uses the standard 8-float layout per vertex:
/// `[pos.x, pos.y, pos.z, norm.x, norm.y, norm.z, uv.u, uv.v]`.
///
/// Writes deformed positions and normals into `out`, which is resized and
/// populated from `bind_verts` as needed. Reusing a caller-owned scratch
/// buffer across frames avoids per-call allocation.
pub(in crate::libs::graphics::renderer3d) fn cpu_skin_submesh(
    bind_verts: &[f32],
    bone_indices: &[[u32; 4]],
    bone_weights: &[[f32; 4]],
    bone_matrices: &[[f32; 16]],
    out: &mut Vec<f32>,
) {
    const FPV: usize = 8; // floats per vertex
    let vert_count = bind_verts.len() / FPV;
    out.resize(bind_verts.len(), 0.0);
    out.copy_from_slice(bind_verts);

    for v in 0..vert_count {
        let base = v * FPV;
        let pos = [bind_verts[base], bind_verts[base + 1], bind_verts[base + 2]];
        let nrm = [
            bind_verts[base + 3],
            bind_verts[base + 4],
            bind_verts[base + 5],
        ];

        let bi = if v < bone_indices.len() {
            bone_indices[v]
        } else {
            [0; 4]
        };
        let bw = if v < bone_weights.len() {
            bone_weights[v]
        } else {
            [0.0; 4]
        };

        let mut sp = [0.0f32; 3];
        let mut sn = [0.0f32; 3];
        let mut total_weight = 0.0f32;

        for i in 0..4 {
            let w = bw[i];
            if w < WEIGHT_EPSILON {
                continue;
            }
            let idx = bi[i] as usize;
            if idx >= bone_matrices.len() {
                continue;
            }
            total_weight += w;
            let m = &bone_matrices[idx]; // column-major [f32; 16]
                                         // Transform position: M * [pos, 1]
            sp[0] += w * (m[0] * pos[0] + m[4] * pos[1] + m[8] * pos[2] + m[12]);
            sp[1] += w * (m[1] * pos[0] + m[5] * pos[1] + m[9] * pos[2] + m[13]);
            sp[2] += w * (m[2] * pos[0] + m[6] * pos[1] + m[10] * pos[2] + m[14]);
            // Transform normal: upper-left 3x3 of M (no translation)
            sn[0] += w * (m[0] * nrm[0] + m[4] * nrm[1] + m[8] * nrm[2]);
            sn[1] += w * (m[1] * nrm[0] + m[5] * nrm[1] + m[9] * nrm[2]);
            sn[2] += w * (m[2] * nrm[0] + m[6] * nrm[1] + m[10] * nrm[2]);
        }

        // When total weight is zero the vertex has no bone influence -- keep the
        // original bind-pose position and normal so it does not collapse to the origin.
        if total_weight < SKIN_WEIGHT_EPSILON {
            continue;
        }

        // Normalize the accumulated result when total weight is not 1.0.
        if (total_weight - 1.0).abs() > SKIN_WEIGHT_EPSILON {
            let inv_w = 1.0 / total_weight;
            sp[0] *= inv_w;
            sp[1] *= inv_w;
            sp[2] *= inv_w;
            sn[0] *= inv_w;
            sn[1] *= inv_w;
            sn[2] *= inv_w;
        }

        // Normalize the skinned normal.
        let len = (sn[0] * sn[0] + sn[1] * sn[1] + sn[2] * sn[2]).sqrt();
        if len > SKIN_WEIGHT_EPSILON {
            sn[0] /= len;
            sn[1] /= len;
            sn[2] /= len;
        }

        out[base] = sp[0];
        out[base + 1] = sp[1];
        out[base + 2] = sp[2];
        out[base + 3] = sn[0];
        out[base + 4] = sn[1];
        out[base + 5] = sn[2];
        // UV (base+6, base+7) unchanged.
    }
}
