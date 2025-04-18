use std::iter::Sum;
use std::ops::{Add, Div, Neg, Sub};
use std::{
    ops::{Index, IndexMut, Mul},
    slice::SliceIndex,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaytracerVector<T, const N: usize> {
    data: [T; N],
}

pub type Vec2f = RaytracerVector<f64, 2>;
pub type Vec3f = RaytracerVector<f64, 3>;
pub type Vec4f = RaytracerVector<f64, 4>;
pub type Vec3i = RaytracerVector<i32, 3>;

impl<T: Copy + From<f64>, const N: usize> RaytracerVector<T, N> {
    pub fn new(value: T) -> Self {
        Self { data: [value; N] }
    }

    pub fn new_with_data(data: [T; N]) -> Self {
        Self { data }
    }

    pub const fn const_new_with_data(data: [T; N]) -> Self {
        Self { data }
    }

    fn apply_op<F>(self, rhs: f64, op: F) -> Self
    where
        F: Fn(T, T) -> T,
    {
        let rhs_t = T::from(rhs);
        let new_data = self.data.map(|x| op(x, rhs_t));
        Self::new_with_data(new_data)
    }

    fn apply_op_vector<F>(self, rhs: RaytracerVector<T, N>, op: F) -> Self
    where
        F: Fn(T, T) -> T,
    {
        let new_data = core::array::from_fn(|i| op(self.data[i], rhs.data[i]));
        Self::new_with_data(new_data)
    }
}

impl<T, const N: usize, Idx> Index<Idx> for RaytracerVector<T, N>
where
    Idx: SliceIndex<[T], Output = T>,
{
    type Output = T;

    fn index(&self, index: Idx) -> &Self::Output {
        self.data.index(index)
    }
}

impl<T, const N: usize, Idx> IndexMut<Idx> for RaytracerVector<T, N>
where
    Idx: SliceIndex<[T], Output = T>,
{
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}

impl<T: Copy + From<f64> + Neg<Output = T>, const N: usize> Neg for RaytracerVector<T, N> {
    type Output = RaytracerVector<T, N>;

    fn neg(self) -> Self::Output {
        let new_data = self.data.map(|x| -x);
        Self::new_with_data(new_data)
    }
}

impl<T, const N: usize> Mul<RaytracerVector<T, N>> for RaytracerVector<T, N>
where
    T: Copy + Mul<Output = T> + From<f64> + Sum,
{
    type Output = T;

    fn mul(self, rhs: RaytracerVector<T, N>) -> Self::Output {
        self.data
            .iter()
            .zip(rhs.data.iter())
            .map(|(a, b)| (*a) * (*b))
            .sum()
    }
}

macro_rules! impl_op_scalar {
    ($op:tt, $trait:ident, $method:ident, $op_fn:expr) => {
        impl<T, const N: usize> $trait<f64> for RaytracerVector<T, N>
        where
            T: Copy + $op<Output = T> + From<f64>,
        {
            type Output = Self;

            fn $method(self, rhs: f64) -> Self::Output {
                self.apply_op(rhs, $op_fn)
            }
        }

        impl<T, const N: usize> $trait<f64> for &RaytracerVector<T, N>
        where
            T: Copy + $op<Output = T> + Into<f64> + From<f64>,
        {
            type Output = RaytracerVector<T, N>;
            fn $method(self, rhs: f64) -> Self::Output {
                self.apply_op(rhs, $op_fn)
            }
        }
    };
}

macro_rules! impl_op_vector {
    ($op:tt, $trait:ident, $method:ident, $op_fn:expr) => {
        impl<T, const N: usize> $trait<RaytracerVector<T, N>> for RaytracerVector<T, N>
        where
            T: Copy + $op<Output = T> + From<f64>,
        {
            type Output = Self;

            fn $method(self, rhs: RaytracerVector<T, N>) -> Self::Output {
                self.apply_op_vector(rhs, $op_fn)
            }
        }

        impl<T, const N: usize> $trait<RaytracerVector<T, N>> for &RaytracerVector<T, N>
        where
            T: Copy + $op<Output = T> + Into<f64> + From<f64>,
        {
            type Output = RaytracerVector<T, N>;
            fn $method(self, rhs: RaytracerVector<T, N>) -> Self::Output {
                self.apply_op_vector(rhs, $op_fn)
            }
        }
    };
}

impl_op_scalar!(Div, Div, div, |x, y| x / y);
impl_op_scalar!(Mul, Mul, mul, |x, y| x * y);

impl_op_vector!(Add, Add, add, |x, y| x + y);
impl_op_vector!(Sub, Sub, sub, |x, y| x - y);

impl<T, const N: usize> RaytracerVector<T, N>
where
    T: Div<Output = T> + Into<f64> + From<f64> + Copy,
{
    pub fn length(&self) -> f64 {
        self.data
            .iter()
            .map(|&x| Into::<f64>::into(x).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    pub fn normalize(&self, scale: Option<f64>) -> Self {
        let length = match scale {
            Some(scale) => self.length() / scale,
            None => self.length(),
        };

        self / length
    }
}

impl<T> RaytracerVector<T, 3>
where
    T: Mul<Output = T> + Sub<Output = T> + From<f64> + Copy,
{
    pub fn cross(self, vector: &RaytracerVector<T, 3>) -> Self {
        let [x1, y1, z1] = self.data;
        let [x2, y2, z2] = vector.data;

        let cx = y1 * z2 - z1 * y2;
        let cy = z1 * x2 - x1 * z2;
        let cz = x1 * y2 - y1 * x2;

        RaytracerVector::new_with_data([cx, cy, cz])
    }

    pub fn cross_static(vector1: &RaytracerVector<T, 3>, vector2: &RaytracerVector<T, 3>) -> Self {
        let [x1, y1, z1] = vector1.data;
        let [x2, y2, z2] = vector2.data;

        let cx = y1 * z2 - z1 * y2;
        let cy = z1 * x2 - x1 * z2;
        let cz = x1 * y2 - y1 * x2;

        RaytracerVector::new_with_data([cx, cy, cz])
    }
}
