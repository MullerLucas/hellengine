use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Neg, Rem, RemAssign, Index, IndexMut};
use std::convert::{AsRef, AsMut};

use crate::vec::Vec3;
use crate::{Vec4};




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

    pub fn rotate(mut self, axis: &Vec3, angle: f32) -> Self {
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

    pub fn translate(mut self, val: Vec3) -> Self {
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

    pub fn from_translate(val: Vec3) -> Self {
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
