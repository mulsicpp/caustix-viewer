use ash::vk;

#[derive(Clone, Copy, Debug, utils::Paramters)]
pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}

impl Extent2D {
    #[inline]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    #[inline]
    pub const fn to_vk(&self) -> vk::Extent2D {
        vk::Extent2D {
            width: self.width,
            height: self.height,
        }
    }

    #[inline]
    pub const fn to_vk_3d(&self) -> vk::Extent3D {
        vk::Extent3D {
            width: self.width,
            height: self.height,
            depth: 1,
        }
    }
}

impl From<(u32, u32)> for Extent2D {
    fn from((width, height): (u32, u32)) -> Self {
        Self { width, height }
    }
}

impl From<[u32; 2]> for Extent2D {
    fn from([width, height]: [u32; 2]) -> Self {
        Self { width, height }
    }
}

impl From<u32> for Extent2D {
    fn from(width: u32) -> Self {
        Self { width, height: 1 }
    }
}

#[derive(Clone, Copy, Debug, utils::Paramters)]
pub struct Extent3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Extent3D {
    #[inline]
    pub const fn new(width: u32, height: u32, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }

    #[inline]
    pub const fn to_vk(&self) -> vk::Extent3D {
        vk::Extent3D {
            width: self.width,
            height: self.height,
            depth: self.depth,
        }
    }
}

impl From<(u32, u32, u32)> for Extent3D {
    fn from((width, height, depth): (u32, u32, u32)) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }
}

impl From<[u32; 3]> for Extent3D {
    fn from([width, height, depth]: [u32; 3]) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }
}

impl From<Extent2D> for Extent3D {
    fn from(Extent2D { width, height }: Extent2D) -> Self {
        Self {
            width,
            height,
            depth: 1,
        }
    }
}

impl From<(u32, u32)> for Extent3D {
    fn from((width, height): (u32, u32)) -> Self {
        Self {
            width,
            height,
            depth: 1,
        }
    }
}

impl From<[u32; 2]> for Extent3D {
    fn from([width, height]: [u32; 2]) -> Self {
        Self {
            width,
            height,
            depth: 1,
        }
    }
}

impl From<u32> for Extent3D {
    fn from(width: u32) -> Self {
        Self {
            width,
            height: 1,
            depth: 1,
        }
    }
}
