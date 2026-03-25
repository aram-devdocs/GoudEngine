//! SQT decomposition, quaternion slerp, and bone matrix blending math.

use super::animation::IDENTITY_MAT4;

/// Decompose a column-major 4x4 matrix into translation, quaternion, and scale.
pub(super) fn decompose_mat4(m: &[f32; 16]) -> ([f32; 3], [f32; 4], [f32; 3]) {
    let t = [m[12], m[13], m[14]];

    let sx = (m[0] * m[0] + m[1] * m[1] + m[2] * m[2]).sqrt();
    let sy = (m[4] * m[4] + m[5] * m[5] + m[6] * m[6]).sqrt();
    let sz = (m[8] * m[8] + m[9] * m[9] + m[10] * m[10]).sqrt();
    let s = [sx, sy, sz];

    let inv_sx = if sx > f32::EPSILON { 1.0 / sx } else { 0.0 };
    let inv_sy = if sy > f32::EPSILON { 1.0 / sy } else { 0.0 };
    let inv_sz = if sz > f32::EPSILON { 1.0 / sz } else { 0.0 };

    let r = [
        m[0] * inv_sx,
        m[1] * inv_sx,
        m[2] * inv_sx,
        m[4] * inv_sy,
        m[5] * inv_sy,
        m[6] * inv_sy,
        m[8] * inv_sz,
        m[9] * inv_sz,
        m[10] * inv_sz,
    ];

    let q = mat3_to_quaternion(&r);
    (t, q, s)
}

/// Convert a 3x3 rotation matrix to a quaternion using Shepperd's method.
///
/// Input layout: `[r00, r10, r20, r01, r11, r21, r02, r12, r22]` (column-major).
pub(super) fn mat3_to_quaternion(r: &[f32; 9]) -> [f32; 4] {
    let (r00, r10, r20) = (r[0], r[1], r[2]);
    let (r01, r11, r21) = (r[3], r[4], r[5]);
    let (r02, r12, r22) = (r[6], r[7], r[8]);

    let trace = r00 + r11 + r22;
    if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        let inv_s = 1.0 / s;
        [
            (r21 - r12) * inv_s,
            (r02 - r20) * inv_s,
            (r10 - r01) * inv_s,
            0.25 * s,
        ]
    } else if r00 > r11 && r00 > r22 {
        let s = (1.0 + r00 - r11 - r22).sqrt() * 2.0;
        let inv_s = 1.0 / s;
        [
            0.25 * s,
            (r01 + r10) * inv_s,
            (r02 + r20) * inv_s,
            (r21 - r12) * inv_s,
        ]
    } else if r11 > r22 {
        let s = (1.0 + r11 - r00 - r22).sqrt() * 2.0;
        let inv_s = 1.0 / s;
        [
            (r01 + r10) * inv_s,
            0.25 * s,
            (r12 + r21) * inv_s,
            (r02 - r20) * inv_s,
        ]
    } else {
        let s = (1.0 + r22 - r00 - r11).sqrt() * 2.0;
        let inv_s = 1.0 / s;
        [
            (r02 + r20) * inv_s,
            (r12 + r21) * inv_s,
            0.25 * s,
            (r10 - r01) * inv_s,
        ]
    }
}

/// Quaternion slerp (spherical linear interpolation).
pub(super) fn quat_slerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let mut dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let mut b = b;

    // Ensure shortest path.
    if dot < 0.0 {
        b = [-b[0], -b[1], -b[2], -b[3]];
        dot = -dot;
    }

    // If nearly identical, use normalised lerp.
    if dot > 0.9995 {
        let mut result = [0.0f32; 4];
        for i in 0..4 {
            result[i] = a[i] + t * (b[i] - a[i]);
        }
        let len = (result[0] * result[0]
            + result[1] * result[1]
            + result[2] * result[2]
            + result[3] * result[3])
            .sqrt();
        if len > f32::EPSILON {
            for v in &mut result {
                *v /= len;
            }
        }
        return result;
    }

    let theta = dot.acos();
    let sin_theta = theta.sin();
    let wa = ((1.0 - t) * theta).sin() / sin_theta;
    let wb = (t * theta).sin() / sin_theta;

    [
        wa * a[0] + wb * b[0],
        wa * a[1] + wb * b[1],
        wa * a[2] + wb * b[2],
        wa * a[3] + wb * b[3],
    ]
}

/// Compose translation + quaternion + scale into a column-major 4x4 matrix.
pub(super) fn compose_tqs(t: [f32; 3], q: [f32; 4], s: [f32; 3]) -> [f32; 16] {
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    let x2 = x + x;
    let y2 = y + y;
    let z2 = z + z;
    let xx = x * x2;
    let xy = x * y2;
    let xz = x * z2;
    let yy = y * y2;
    let yz = y * z2;
    let zz = z * z2;
    let wx = w * x2;
    let wy = w * y2;
    let wz = w * z2;

    [
        s[0] * (1.0 - yy - zz),
        s[0] * (xy + wz),
        s[0] * (xz - wy),
        0.0,
        s[1] * (xy - wz),
        s[1] * (1.0 - xx - zz),
        s[1] * (yz + wx),
        0.0,
        s[2] * (xz + wy),
        s[2] * (yz - wx),
        s[2] * (1.0 - xx - yy),
        0.0,
        t[0],
        t[1],
        t[2],
        1.0,
    ]
}

/// Blend bone matrices using SQT decomposition and quaternion slerp.
pub(super) fn slerp_bone_matrices(a: &[[f32; 16]], b: &[[f32; 16]], t: f32) -> Vec<[f32; 16]> {
    a.iter()
        .zip(b.iter())
        .map(|(ma, mb)| {
            let (ta, qa, sa) = decompose_mat4(ma);
            let (tb, qb, sb) = decompose_mat4(mb);

            let t_blend = [
                ta[0] + t * (tb[0] - ta[0]),
                ta[1] + t * (tb[1] - ta[1]),
                ta[2] + t * (tb[2] - ta[2]),
            ];
            let q_blend = quat_slerp(qa, qb, t);
            let s_blend = [
                sa[0] + t * (sb[0] - sa[0]),
                sa[1] + t * (sb[1] - sa[1]),
                sa[2] + t * (sb[2] - sa[2]),
            ];

            compose_tqs(t_blend, q_blend, s_blend)
        })
        .collect()
}

pub(super) fn identity_matrices(count: usize) -> Vec<[f32; 16]> {
    vec![IDENTITY_MAT4; count]
}
