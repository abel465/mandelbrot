#[cfg(not(target_arch = "spirv"))]
use bytemuck::NoUninit;
use glam::*;

pub mod shader;

#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(NoUninit))]
#[repr(C)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn aspect_ratio(self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn as_vec2(self) -> Vec2 {
        vec2(self.width as f32, self.height as f32)
    }

    pub fn as_uvec2(self) -> UVec2 {
        uvec2(self.width, self.height)
    }
}

impl From<UVec2> for Size {
    fn from(v: UVec2) -> Self {
        Self {
            width: v.x,
            height: v.y,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(NoUninit))]
#[repr(C)]
pub struct Bool(pub u32);

impl From<bool> for Bool {
    fn from(b: bool) -> Self {
        Self(b as u32)
    }
}

impl From<Bool> for bool {
    fn from(b: Bool) -> bool {
        b.0 != 0
    }
}
