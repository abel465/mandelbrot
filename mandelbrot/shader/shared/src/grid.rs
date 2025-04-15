use glam::*;

#[derive(Clone, Copy)]
pub struct GridRef<'a, T> {
    size: UVec2,
    buffer: &'a [T],
}

impl<'a, T: Copy> GridRef<'a, T> {
    pub fn new(size: UVec2, buffer: &'a [T]) -> Self {
        Self { size, buffer }
    }

    pub fn get(&self, p: UVec2) -> T {
        self.buffer[(p.y * self.size.x + p.x) as usize]
    }
}

pub struct GridRefMut<'a, T> {
    size: UVec2,
    buffer: &'a mut [T],
}

impl<'a, T: Copy> GridRefMut<'a, T> {
    pub fn new(size: UVec2, buffer: &'a mut [T]) -> Self {
        Self { size, buffer }
    }

    pub fn as_ref(&self) -> GridRef<'_, T> {
        GridRef::new(self.size, self.buffer)
    }

    pub fn get(&self, p: UVec2) -> T {
        self.buffer[(p.y * self.size.x + p.x) as usize]
    }

    pub fn set(&mut self, p: UVec2, value: T) {
        self.buffer[(p.y * self.size.x + p.x) as usize] = value;
    }

    pub fn swap(&mut self, a: UVec2, b: UVec2) {
        let tmp = self.get(a);
        self.set(a, self.get(b));
        self.set(b, tmp);
    }
}

#[cfg(not(target_arch = "spirv"))]
pub struct Grid<T> {
    pub size: UVec2,
    pub buffer: Vec<T>,
}

#[cfg(not(target_arch = "spirv"))]
impl<T> Grid<T>
where
    T: Default + Clone + Copy,
{
    pub fn new(size: UVec2) -> Self {
        Self {
            size,
            buffer: vec![Default::default(); (size.x * size.y) as usize],
        }
    }

    pub fn as_ref(&self) -> GridRef<'_, T> {
        GridRef::new(self.size, &self.buffer)
    }

    pub fn as_ref_mut(&mut self) -> GridRefMut<'_, T> {
        GridRefMut::new(self.size, &mut self.buffer)
    }

    pub fn resize(&mut self, size: UVec2) {
        self.size = size;
        let length = (size.x * size.y) as usize;
        if length > self.buffer.len() {
            self.buffer.resize(length, Default::default());
        }
    }

    pub fn get(&self, p: UVec2) -> T {
        self.as_ref().get(p)
    }

    pub fn set(&mut self, p: UVec2, value: T) {
        self.as_ref_mut().set(p, value)
    }

    pub fn swap(&mut self, a: UVec2, b: UVec2) {
        self.as_ref_mut().swap(a, b)
    }
}
