use std::{
    ops::{Add, Deref, Div, Index, IndexMut, Mul, Neg, RangeTo, Sub},
    rc::Rc,
};

pub type GenericResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub struct DataSource<T: Copy>(Rc<T>);
impl<T: Copy> Deref for DataSource<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
impl<T: Copy> DataSource<T> {
    pub fn new(v: T) -> Self {
        Self(Rc::new(v))
    }
    /// Sets the value of the data source/sink to the given value.
    pub fn set(&self, mut v: T) {
        unsafe {
            std::mem::swap(&mut v, &mut *(self.0.as_ref() as *const _ as *mut _));
            drop(v);
        }
    }
    pub fn make_sink(&self) -> DataSink<T> {
        DataSink(self.0.clone())
    }
}

#[derive(Debug, Clone)]
pub struct DataSink<T: Copy>(Rc<T>);
impl<T: Copy> Deref for DataSink<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

/// Non-heap-allocated vector of `T` with maximum length `N`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NoAllocVec<T, const N: usize> {
    len: usize,
    data: [T; N],
}
impl<T, const N: usize> NoAllocVec<T, N> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn new() -> Self {
        Self {
            len: 0,
            data: unsafe { std::mem::zeroed() },
        }
    }
    pub fn try_push(&mut self, mut value: T) -> Result<(), T> {
        if self.len == N {
            return Err(value);
        }

        std::mem::swap(&mut self.data[self.len], &mut value);
        std::mem::forget(value);
        self.len += 1;

        Ok(())
    }
    pub fn extend(&mut self, values: impl Iterator<Item = T>) {
        for value in values {
            self.push(value);
        }
    }
    pub fn push(&mut self, value: T) {
        self.try_push(value)
            .map_err(|_| "cannot push into full StackVec")
            .unwrap()
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        let mut value = unsafe { std::mem::zeroed() };
        std::mem::swap(&mut self.data[self.len], &mut value);

        Some(value)
    }
}

impl<T, I, const N: usize> Index<I> for NoAllocVec<T, N>
where
    [T]: Index<RangeTo<usize>>,
    <[T] as Index<RangeTo<usize>>>::Output: Index<I>,
{
    type Output = <<[T] as Index<RangeTo<usize>>>::Output as Index<I>>::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.data[..self.len], index)
    }
}

impl<T, I, const N: usize> IndexMut<I> for NoAllocVec<T, N>
where
    [T]: IndexMut<RangeTo<usize>>,
    <[T] as Index<RangeTo<usize>>>::Output: IndexMut<I>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.data[..self.len], index)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Vec2I(pub i32, pub i32);
impl Add for Vec2I {
    type Output = Self;
    fn add(self, Self(x1, y1): Self) -> Self::Output {
        let Self(x0, y0) = self;
        Self(x0 + x1, y0 + y1)
    }
}
impl Sub for Vec2I {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}
impl Mul<i32> for Vec2I {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        let Self(x, y) = self;

        Self(x * rhs, y * rhs)
    }
}
impl Neg for Vec2I {
    type Output = Self;
    fn neg(self) -> Self::Output {
        let Self(x, y) = self;
        Self(-x, -y)
    }
}
impl Div<i32> for Vec2I {
    type Output = Self;
    fn div(self, rhs: i32) -> Self::Output {
        let Self(x, y) = self;
        Self(x / rhs, y / rhs)
    }
}
impl From<(i32, i32)> for Vec2I {
    fn from((x, y): (i32, i32)) -> Self {
        Self(x, y)
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct RectI {
    pub pos: Vec2I,
    pub dim: Vec2I,
}
impl RectI {
    pub fn aspect(&self) -> f32 {
        let Vec2I(width, height) = self.dim;
        return height as f32 / width as f32;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2(f64, f64);
impl Add for Vec2 {
    type Output = Self;
    fn add(self, Self(x1, y1): Self) -> Self::Output {
        let Self(x0, y0) = self;
        Self(x0 + x1, y0 + y1)
    }
}
impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}
impl Mul<f64> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        let Self(x, y) = self;

        Self(x * rhs, y * rhs)
    }
}
impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        let Self(x, y) = self;
        Self(-x, -y)
    }
}
impl Div<f64> for Vec2 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        let Self(x, y) = self;
        Self(x / rhs, y / rhs)
    }
}
