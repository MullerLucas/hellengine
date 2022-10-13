use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Neg, Index, IndexMut};

use crate::vec::Vec3;
use crate::Vec4;




macro_rules! impl_matrix_common {
    ($mat:ident : $vec:ident : $scalar:ident => $mat_size:literal : { $($field_idx:literal : $field:ident),+ }) => {
        #[repr(C)]
        pub struct $mat {
            $(pub $field: $vec),+
        }

        // constants
        // ---------
        impl $mat {
            pub const IDENTITY: $mat = $mat::from_cols(
                $($vec::with_one_at($field_idx)),+
            );
        }

        // creators
        // --------
        impl $mat {
            pub const fn from_cols($($field: $vec),+) -> Self {
                Self {
                    $($field),+
                }
            }
        }

        // add operations
        // --------------
        // mat + mat = mat
        impl Add<Self> for $mat {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self::from_cols(
                    $(self.$field.add(rhs.$field)),+
                )
            }
        }

        // mat += mat
        impl AddAssign<Self> for $mat {
            fn add_assign(&mut self, rhs: Self) {
                $(self.$field.add_assign(rhs.$field));+
            }
        }

        // sub operations
        // --------------
        // mat - mat = mat
        impl Sub<Self> for $mat {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self {
                    $($field: self.$field.sub(rhs.$field)),+
                }
            }
        }

        // mat -= mat
        impl SubAssign<Self> for $mat {
            fn sub_assign(&mut self, rhs: Self) {
                $(self.$field.sub_assign(rhs.$field));+
            }
        }

        // mul operations
        // --------------
        impl Mul<$scalar> for $mat {
            type Output = Self;
            fn mul(self, rhs: $scalar) -> Self {
                Self {
                    $($field: self.$field.mul(rhs)),+
                }
            }
        }

        impl MulAssign<$scalar> for $mat {
            fn mul_assign(&mut self, rhs: $scalar) {
                $(self.$field.mul_assign(rhs));+
            }
        }

        impl Mul<$vec> for $mat {
            type Output = $vec;
            fn mul(self, rhs: $vec) -> Self::Output {
                self.mul_vec(&rhs)
            }
        }

        impl Mul<Self> for $mat {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self {
                self.mul_mat(&rhs)
            }
        }

        // custom operations
        // -----------------
        impl $mat {
            // #[inline]
            pub fn mul_vec(&self, rhs: &$vec) -> $vec {
                let mut result = $vec::zero();
                $(result.add_assign(self.$field.mul_scalar(rhs.$field));)+
                result
            }

            // #[inline]
            pub fn mul_mat(&self, rhs: &Self) -> Self {
                Self {
                    $($field: self.mul_vec(&rhs.$field)),+
                }
            }
        }

        // indexers
        // --------
        impl Index<usize> for $mat {
            type Output = $vec;
            fn index(&self, index: usize) -> &Self::Output {
                match index {
                    0 => &self.x,
                    1 => &self.y,
                    2 => &self.z,
                    3 => &self.w,
                    _ => panic!("index out of bounds!"),
                }
            }
        }

        impl IndexMut<usize> for $mat {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                match index {
                    0 => &mut self.x,
                    1 => &mut self.y,
                    2 => &mut self.z,
                    3 => &mut self.w,
                    _ => panic!("index out of bounds!"),
                }
            }
        }

    };
}

macro_rules! impl_matrix_signed{
    ($mat:ident : $vec:ident : $scalar:ident => $mat_size:literal : { $($field_idx:literal : $field:ident),+ }) => {
        // neg operations
        // --------------
        impl Neg for $mat {
            type Output = Self;
            fn neg(self) -> Self::Output {
                Self {
                    $($field: self.$field.neg()),+
                }
            }
        }
    };
}

impl_matrix_common!(Mat4: Vec4: f32 => 2: { 0:x, 1:y, 2:z, 3:w });
impl_matrix_signed!(Mat4: Vec4: f32 => 2: { 0:x, 1:y, 2:z, 3:w });


impl Mat4 {
    pub fn inverse(self) -> Self {
         let (m00, m01, m02, m03) = self.x.into();
        let (m10, m11, m12, m13) = self.y.into();
        let (m20, m21, m22, m23) = self.z.into();
        let (m30, m31, m32, m33) = self.w.into();

        let coef00 = m22 * m33 - m32 * m23;
        let coef02 = m12 * m33 - m32 * m13;
        let coef03 = m12 * m23 - m22 * m13;

        let coef04 = m21 * m33 - m31 * m23;
        let coef06 = m11 * m33 - m31 * m13;
        let coef07 = m11 * m23 - m21 * m13;

        let coef08 = m21 * m32 - m31 * m22;
        let coef10 = m11 * m32 - m31 * m12;
        let coef11 = m11 * m22 - m21 * m12;

        let coef12 = m20 * m33 - m30 * m23;
        let coef14 = m10 * m33 - m30 * m13;
        let coef15 = m10 * m23 - m20 * m13;

        let coef16 = m20 * m32 - m30 * m22;
        let coef18 = m10 * m32 - m30 * m12;
        let coef19 = m10 * m22 - m20 * m12;

        let coef20 = m20 * m31 - m30 * m21;
        let coef22 = m10 * m31 - m30 * m11;
        let coef23 = m10 * m21 - m20 * m11;

        let fac0 = Vec4::new(coef00, coef00, coef02, coef03);
        let fac1 = Vec4::new(coef04, coef04, coef06, coef07);
        let fac2 = Vec4::new(coef08, coef08, coef10, coef11);
        let fac3 = Vec4::new(coef12, coef12, coef14, coef15);
        let fac4 = Vec4::new(coef16, coef16, coef18, coef19);
        let fac5 = Vec4::new(coef20, coef20, coef22, coef23);

        let vec0 = Vec4::new(m10, m00, m00, m00);
        let vec1 = Vec4::new(m11, m01, m01, m01);
        let vec2 = Vec4::new(m12, m02, m02, m02);
        let vec3 = Vec4::new(m13, m03, m03, m03);

        let inv0 = vec1.mul(fac0).sub(vec2.mul(fac1)).add(vec3.mul(fac2));
        let inv1 = vec0.mul(fac0).sub(vec2.mul(fac3)).add(vec3.mul(fac4));
        let inv2 = vec0.mul(fac1).sub(vec1.mul(fac3)).add(vec3.mul(fac5));
        let inv3 = vec0.mul(fac2).sub(vec1.mul(fac4)).add(vec2.mul(fac5));

        let sign_a = Vec4::new(1.0, -1.0, 1.0, -1.0);
        let sign_b = Vec4::new(-1.0, 1.0, -1.0, 1.0);

        let inverse = Self::from_cols(
            inv0.mul(sign_a),
            inv1.mul(sign_b),
            inv2.mul(sign_a),
            inv3.mul(sign_b),
        );

        let col0 = Vec4::new(
            inverse.x.x,
            inverse.y.x,
            inverse.z.x,
            inverse.w.x,
        );

        let dot0 = self.x.mul(col0);
        let dot1 = dot0.x + dot0.y + dot0.z + dot0.w;

        let rcp_det = dot1.recip();
        inverse.mul(rcp_det)
    }
}

// oder: Mt * Mr * Ms * V
impl Mat4 {
    // TODO:
    pub fn scale(mut self, val: &[f32; 3]) -> Self {
        // self[0][0] *= val[0];
        // self[1][1] *= val[1];
        // self[2][2] *= val[2];
        self.x[0] *= val[0];
        self.y[1] *= val[1];
        self.y[2] *= val[2];
        // self[0] *= val[0];
        // self[1].y *= val[1];
        // self[2].z *= val[2];
        self
    }

    pub fn rotate(self, axis: &Vec3, angle: f32) -> Self {
        let cos = angle.cos();
        let one_minus_cos = 1.0 - cos;
        let sin = angle.sin();

        let x = Vec4::new(
            cos + axis.x * axis.x * one_minus_cos,
            axis.y * axis.x * one_minus_cos + axis.z * sin,
            axis.z * axis.x * one_minus_cos - axis.y * sin,
            0.0
        );

        let y = Vec4::new(
            axis.x * axis.y * one_minus_cos - axis.z * sin,
            cos + axis.y * axis.y * one_minus_cos,
            axis.z * axis.y * one_minus_cos + axis.x * sin,
            0.0
        );

        let z = Vec4::new(
            axis.x * axis.z * one_minus_cos + axis.y * sin,
            axis.y * axis.z * one_minus_cos - axis.x * sin,
            cos + axis.z * axis.z * one_minus_cos,
            0.0
        );

        let w = Vec4::new(0.0, 0.0, 0.0, 1.0);

        let scale_mat = Mat4::from_cols(x, y, z, w);
        self * scale_mat
    }

    pub fn translate(mut self, val: &Vec3) -> Self {
        let val = Vec4::new(val.x, val.y, val.z, 0.0);
        self.w += val;
        self
    }

    pub fn from_scale(val: &[f32; 3]) -> Self {
        Self::IDENTITY.scale(val)
    }

    pub fn from_rotation(axis: &Vec3, angle: f32) -> Self {
        Self::IDENTITY.rotate(axis, angle)
    }

    pub fn from_translate(val: &Vec3) -> Self {
        Self::IDENTITY.translate(val)
    }
}

// https://vincent-p.github.io/posts/vulkan_perspective_matrix/
impl Mat4 {
    pub fn from_perspective_rh_corners(x_left: f32, x_right: f32, y_top: f32, y_bottom: f32, z_near: f32, z_far: f32) -> Self {
        Self::from_perspective_rh_dimensions(
            x_right - x_left,
            y_top - y_bottom,
            z_near, z_far
        )
    }

    pub fn from_perspective_rh_dimensions(width: f32, height: f32, z_near: f32, z_far: f32) -> Self {
        let x = Vec4::new(
            (2.0 * z_near) / width,
            0.0,
            0.0,
            0.0
        );

        let y = Vec4::new(
            0.0,
            (2.0 * z_near) / height,
            0.0,
            0.0
        );

        let z = Vec4::new(
            0.0,
            0.0,
            z_near / (z_far - z_near),
            -1.0
        );

        let w = Vec4::new(
            0.0,
            0.0,
            (z_near * z_far) / (z_far - z_near),
            0.0
        );

        Self::from_cols(x, y, z, w)
    }

    pub fn from_perspective_rh(vertical_fov: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
        let fov_rad = vertical_fov * 2.0 * std::f32::consts::PI / 360.0;
        let focal_length = 1.0 / (fov_rad / 2.0).tan();

        let x = Vec4::new(
            focal_length / aspect_ratio,
            0.0,
            0.0,
            0.0
        );

        let y = Vec4::new(
            0.0,
            -focal_length,
            0.0,
            0.0
        );

        let z = Vec4::new(
            0.0,
            0.0,
            z_near / (z_far - z_near),
            -1.0
        );

        let w = Vec4::new(
            0.0,
            0.0,
            (z_near * z_far) / (z_far - z_near),
            0.0
        );

        Self::from_cols(x, y, z, w)
    }
}

impl Mat4 {
    pub fn look_at_rh(pos: &Vec3, direction: &Vec3, up: &Vec3) -> Self {
        let cam_right = direction.cross(up).unit();
        let cam_up = cam_right.cross(direction);

        let cam_dir = direction.neg();

        let rotation = Self::from_cols(
            Vec4::new(cam_right.x, cam_right.y, cam_right.z, 0.0),
            Vec4::new(cam_up.x, cam_up.y, cam_up.z, 0.0),
            Vec4::new(cam_dir.x, cam_dir.y, cam_dir.z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0)
        );

        let translation = Self::from_translate(pos);

        (translation * rotation).inverse()
    }
}
