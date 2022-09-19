use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Neg, Rem, RemAssign, Index, IndexMut};
use std::convert::{AsRef, AsMut};



#[macro_export]
macro_rules! impl_vector_common {
    ($vec:ident : $scalar:ident => $vec_size:literal : { $($field_idx:literal : $field:ident),+ }) => {
        #[derive(Clone, Copy)]
        pub struct $vec {
            $(pub $field: $scalar),+
        }


        // constants
        // ---------
        impl $vec {
            pub const SCALAR_ZERO: $scalar = 0 as $scalar;
            pub const SCALAR_ONE: $scalar = 1 as $scalar;
        }

        // creators
        // --------
        impl $vec {
            pub const fn new($($field: $scalar),+) -> Self {
                Self {
                    $($field),+
                }
            }

            pub const fn zero() -> Self {
                Self {
                    $($field: Self::SCALAR_ZERO),+
                }
            }

            pub const fn from_array(val: [$scalar; $vec_size]) -> Self {
                Self {
                    $($field: val[$field_idx]),+
                }
            }

            pub fn from_slice(val: &[$scalar]) -> Self {
                Self {
                    $($field: val[$field_idx].clone()),+
                }
            }

            pub const fn with_one_at(idx: usize) -> Self {
                $(let $field = if $field_idx == idx { Self::SCALAR_ONE } else { Self::SCALAR_ZERO };)+

                Self {
                    $($field),+
                }
            }
        }

        // conversions
        // -----------
        impl AsRef<[$scalar; $vec_size]> for $vec {
            fn as_ref(&self) -> &[$scalar; $vec_size] {
                unsafe {
                    &*(self as *const $vec as *const [$scalar; $vec_size])
                }
            }
        }

        impl AsMut<[$scalar; $vec_size]> for $vec {
            fn as_mut(&mut self) -> &mut [$scalar; $vec_size] {
                unsafe {
                    &mut *(self as *mut $vec as *mut [$scalar; $vec_size])
                }
            }
        }

        // add operations
        // --------------
        // vec + scalar + vec
        impl Add<$scalar> for $vec {
            type Output = Self;

            fn add(self, rhs: $scalar) -> Self::Output {
                Self {
                    $($field: self.$field.add(rhs)),+
                }
            }
        }

        // scalar + vec = vec
        impl Add<$vec> for $scalar {
            type Output = $vec;

            fn add(self, rhs: $vec) -> Self::Output {
                $vec {
                    $($field: self.add(rhs.$field)),+
                }
            }
        }

        // vec += scalar
        impl AddAssign<$scalar> for $vec {
            fn add_assign(&mut self, rhs: $scalar) {
                $(self.$field.add_assign(rhs));+
            }
        }

        // vec + vec = vec
        impl Add<$vec> for $vec {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field.add(rhs.$field)),+
                }
            }
        }

        // vec += vec
        impl AddAssign<$vec> for $vec {
            fn add_assign(&mut self, rhs: Self) {
                $(self.$field.add_assign(rhs.$field));+
            }
        }

        // sub operations
        // --------------
        // vec - scalar = vec
        impl Sub<$scalar> for $vec {
            type Output = Self;
            fn sub (self, rhs: $scalar) -> Self {
                Self {
                    $($field: self.$field.sub(rhs)),+
                }
            }
        }

        // vec -= scalar
        impl SubAssign<$scalar> for $vec {
            fn sub_assign(&mut self, rhs: $scalar) {
                $(self.$field.sub_assign(rhs));+
            }
        }

        // vec - vec = vec
        impl Sub<Self> for $vec {
            type Output = Self;
            fn sub (self, rhs: Self) -> Self {
                Self {
                    $($field: self.$field.sub(rhs.$field)),+
                }
            }
        }

        // vec -= vec
        impl SubAssign<Self> for $vec {
            fn sub_assign(&mut self, rhs: Self) {
                $(self.$field.sub_assign(rhs.$field));+
            }
        }

        // mul operations
        // --------------
        // vec * scalar = vec
        impl Mul<$scalar> for $vec {
            type Output = Self;

            fn mul(self, rhs: $scalar) -> Self::Output {
                Self {
                    $($field: self.$field.mul(rhs)),+
                }
            }
        }

        // scalar * vec = vec
        impl Mul<$vec> for $scalar {
            type Output = $vec;

            fn mul(self, rhs: $vec) -> Self::Output {
                $vec {
                    $($field: self.mul(rhs.$field)),+
                }
            }
        }

        // vec *= scalar
        impl MulAssign<$scalar> for $vec {
            fn mul_assign(&mut self, rhs: $scalar) {
                $(self.$field.mul_assign(rhs));+
            }
        }

        // vec * vec = vec
        impl Mul<Self> for $vec {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field.mul(rhs.$field)),+
                }
            }
        }
        impl Add<Self> for &$vec{
            type Output = $vec;
            fn add(self, rhs: Self) -> Self::Output {
                $vec {
                    $($field: self.$field + rhs.$field),+
                }
            }
        }

        // vec *= vec
        impl MulAssign<Self> for $vec {
            fn mul_assign(&mut self, rhs: Self) {
                $(self.$field.mul_assign(rhs.$field));+
            }
        }

        impl Mul<Self> for &$vec {
            type Output = $vec;
            fn mul(self, rhs: &$vec) -> Self::Output {
                $vec {
                    $($field: self.$field.mul(rhs.$field)),+
                }
            }
        }


        // div operations
        // --------------
        // vec / scalar = vec
        impl Div<$scalar> for $vec {
            type Output = Self;
            fn div(self, rhs: $scalar) -> Self::Output {
                Self {
                    $($field: self.$field.div(rhs)),+
                }
            }
        }

        // vec /= scalar
        impl DivAssign<$scalar> for $vec {
            fn div_assign(&mut self, rhs: $scalar) {
                $(self.$field.div_assign(rhs));+
            }
        }

        // vec / vec = vec
        impl Div<Self> for $vec {
            type Output = Self;
            fn div(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field.div(rhs.$field)),+
                }
            }
        }

        // vec /= vec
        impl DivAssign<Self> for $vec {
            fn div_assign(&mut self, rhs: Self) {
                $(self.$field.div_assign(rhs.$field));+
            }
        }

        // rem operations
        // --------------
        // vec % scalar = scalar
        impl Rem<$scalar> for $vec {
            type Output = Self;
            fn rem(self, rhs: $scalar) -> Self {
                Self {
                    $( $field: self.$field.rem(rhs) ),+
                }
            }
        }

        // vec % vec = scalar
        impl Rem<Self> for $vec {
            type Output = Self;
            fn rem(self, rhs: Self) -> Self {
                Self {
                    $($field: self.$field.rem(rhs.$field)),+
                }
            }
        }

        impl RemAssign<Self> for $vec {
            fn rem_assign(&mut self, rhs: Self) {
                $(self.$field.rem_assign(rhs.$field));+
            }
        }

        // index operations
        // ----------------
        // vec[idx] = &scalar
        impl Index<usize> for $vec {
            type Output = $scalar;
            fn index(&self, index: usize) -> &Self::Output {
                match index {
                    $($field_idx => &self.$field,)+
                    _ => panic!("index out of bounds!"),
                }
            }
        }

        // vec[idx] = &mut scalar
        impl IndexMut<usize> for $vec {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                match index {
                    $($field_idx => &mut self.$field,)+
                    _ => panic!("index out of bounds!"),
                }
            }
        }

        // custom operatiosn
        // -----------------
        impl $vec {
            pub fn mul_scalar(&self, rhs: $scalar) -> Self {
                Self {
                    $($field: self.$field.mul(rhs)),+
                }
            }

            pub fn mul_vec(&self, rhs: &Self) -> Self {
                Self {
                    $($field: self.$field.mul(rhs.$field)),+
                }
            }

            pub fn mul_assign_vec(&mut self, rhs: &Self) {
                $(self.$field.mul_assign(rhs.$field));+
            }
        }

        // additionsl perations
        // --------------------
        impl $vec {
            // returns cos of angle
            // 0 -> angle = 90 deg
            // 1 -> angle =  0 deg
            pub fn dot(&self, rhs: Self) -> $scalar {
                let mut result = Self::SCALAR_ZERO;
                $(result += self.$field * rhs.$field;)+
                result
            }
        }
    };
}

macro_rules! impl_vector_signed {
    ($vec:ident : $scalar:ident => $vec_size:literal : { $($field_idx:literal : $field:ident),+ }) => {
        // neg operations
        // --------------
        // -vec
        impl Neg for $vec {
            type Output = Self;
            fn neg (self) -> Self::Output {
                Self {
                    $($field: self.$field.neg()),+
                }
            }
        }
    };
}

macro_rules! impl_vector_float {
    ($vec:ident : $scalar:ident => $vec_size:literal : { $($field_idx:literal : $field:ident),+ }) => {
        // additionsl perations
        // --------------------
        impl $vec {
            pub fn mag(&self) -> $scalar {
                let mut tmp = 0.0;
                $(tmp.add_assign(self.$field * self.$field);)+
                $scalar::sqrt(tmp)
            }

            pub fn unit(self) -> $vec {
                let mag = self.mag();
                self.div(mag)
            }

            pub fn unit_assign(&mut self) {
                let mag = self.mag();
                self.div_assign(mag);
            }
        }
    };
}




impl_vector_common!(Vec2: f32 => 2: { 0:x, 1:y });
impl_vector_signed!(Vec2: f32 => 2: { 0:x, 1:y });
impl_vector_float! (Vec2: f32 => 2: { 0:x, 1:y });

impl_vector_common!(Vec3: f32 => 3: { 0:x, 1:y, 2:z });
impl_vector_signed!(Vec3: f32 => 3: { 0:x, 1:y, 2:z });
impl_vector_float! (Vec3: f32 => 3: { 0:x, 1:y, 2:z });

impl_vector_common!(Vec4: f32 => 4: { 0:x, 1:y, 2:z, 3:w });
impl_vector_signed!(Vec4: f32 => 4: { 0:x, 1:y, 2:z, 3:w });
impl_vector_float! (Vec4: f32 => 4: { 0:x, 1:y, 2:z, 3:w });

impl_vector_common!(IVec4: i32 => 4: { 0:x, 1:y, 2:z, 3:w });
impl_vector_signed!(IVec4: i32 => 4: { 0:x, 1:y, 2:z, 3:w });

impl_vector_common!(UVec4: u32 => 4: { 0:x, 1:y, 2:z, 3:w });




impl From<(f32, f32, f32, f32)> for Vec4 {
    fn from(val: (f32, f32, f32, f32)) -> Self {
        Self::new(val.0, val.1, val.2, val.3)
    }
}

impl From<Vec4> for (f32, f32, f32, f32) {
    fn from(val: Vec4) -> Self {
        (val.x, val.y, val.z, val.w)
    }
}

impl Vec3 {
    pub fn cross(&self, rhs: &Self) -> Self {
        Self::new(
            (self.y * rhs.z) - (self.z - rhs.y),
            (self.z * rhs.x) - (self.x - rhs.z),
            (self.x * rhs.y) - (self.y - rhs.x)
        )
    }
}
