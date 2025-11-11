use std::{
    num::NonZero,
    ptr::{NonNull, copy_nonoverlapping, slice_from_raw_parts, slice_from_raw_parts_mut},
};

use ash::vk;
use vk_mem::Alloc;

use crate::{Context, MemoryUsage, Recording, VkHandle};

use utils::{AnyRange, Build, Buildable, ToRegion};

type DeviceRegion = utils::Region<vk::DeviceSize>;

pub type BufferUsage = vk::BufferUsageFlags;

#[derive(cvk_macros::VkHandle)]
pub struct Buffer<T: Copy> {
    handle: vk::Buffer,
    allocation: vk_mem::Allocation,

    count: vk::DeviceSize,
    mapped_data: Option<NonNull<T>>,
}

impl<T: Copy> Buffer<T> {
    pub fn count(&self) -> vk::DeviceSize {
        self.count
    }

    pub fn size(&self) -> vk::DeviceSize {
        self.count * size_of::<T>() as vk::DeviceSize
    }

    pub fn mapped(&self) -> Option<&[T]> {
        Some(unsafe { &*slice_from_raw_parts(self.mapped_data?.as_ptr(), self.count as usize) })
    }

    pub fn mapped_mut(&mut self) -> Option<&mut [T]> {
        Some(unsafe {
            &mut *slice_from_raw_parts_mut(self.mapped_data?.as_ptr(), self.count as usize)
        })
    }
}

impl<T: Copy> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe {
            Context::get()
                .allocator()
                .destroy_buffer(self.handle, &mut self.allocation);
        }
    }
}

impl<T: Copy> Buildable for Buffer<T> {
    type Builder<'a> = BufferBuilder<'a, T> where T: 'a;
}



pub struct BufferRegion<'a, T: Copy> {
    buffer: &'a Buffer<T>,
    region: DeviceRegion,
}

impl<'a, T: Copy> From<&'a Buffer<T>> for BufferRegion<'a, T> {
    fn from(buffer: &'a Buffer<T>) -> Self {
        BufferRegion {
            buffer,
            region: DeviceRegion::new(0, buffer.count),
        }
    }
}

impl Recording {
    pub fn copy_buffer<'a, T: 'a + Copy>(
        &'a self,
        src: impl Into<BufferRegion<'a, T>>,
        dst: impl Into<BufferRegion<'a, T>>,
    ) {
        let BufferRegion {
            buffer: src_buffer,
            region:
                DeviceRegion {
                    offset: src_offset,
                    count: src_count,
                },
        } = src.into();
        let BufferRegion {
            buffer: dst_buffer,
            region:
                DeviceRegion {
                    offset: dst_offset,
                    count: dst_count,
                },
        } = dst.into();

        let size = src_count.min(dst_count) * size_of::<T>() as vk::DeviceSize;

        let raw_region = vk::BufferCopy::default()
            .size(size)
            .src_offset(src_offset * size_of::<T>() as vk::DeviceSize)
            .dst_offset(dst_offset * size_of::<T>() as vk::DeviceSize);

        unsafe {
            Context::get_device().cmd_copy_buffer(
                self.handle(),
                src_buffer.handle,
                dst_buffer.handle,
                &[raw_region],
            );
        }
    }

    pub fn copy_buffer_regions<T: Copy>(
        &self,
        src_buffer: &Buffer<T>,
        dst_buffer: &Buffer<T>,
        regions: &[(AnyRange<vk::DeviceSize>, AnyRange<vk::DeviceSize>)],
    ) {
        let raw_regions: Vec<_> = regions
            .iter()
            .map(|(src, dst)| {
                let src = src.clone().to_region(src_buffer.count);
                let dst = dst.clone().to_region(dst_buffer.count);
                vk::BufferCopy::default()
                    .size(src.count.min(dst.count) * size_of::<T>() as vk::DeviceSize)
                    .src_offset(src.offset * size_of::<T>() as vk::DeviceSize)
                    .dst_offset(dst.offset * size_of::<T>() as vk::DeviceSize)
            })
            .collect();

        unsafe {
            Context::get_device().cmd_copy_buffer(
                self.handle(),
                src_buffer.handle,
                dst_buffer.handle,
                &raw_regions,
            );
        }
    }
}



#[derive(Clone, Debug, utils::Paramters)]
pub struct BufferBuilder<'a, T> {
    #[no_param]
    count: NonZero<vk::DeviceSize>,
    #[no_param]
    data: Option<&'a [T]>,
    #[flag]
    usage: BufferUsage,
    memory_usage: MemoryUsage,
    mapped_data: bool,
}

impl<'a, T> BufferBuilder<'a, T> {
    pub fn count(mut self, size: impl Into<vk::DeviceSize>) -> Self {
        self.count = NonZero::new(size.into()).expect("Buffer size needs to be greater than zero");
        self
    }

    pub fn data(mut self, size: &'a [T]) -> Self {
        self.data = Some(size.into());
        self
    }

    pub fn staging_buffer(self) -> Self {
        self.usage(BufferUsage::TRANSFER_SRC)
            .memory_usage(MemoryUsage::PreferHost)
            .mapped_data(true)
    }
}

impl<T> Default for BufferBuilder<'_, T> {
    fn default() -> Self {
        Self {
            count: unsafe { NonZero::new_unchecked(1) },
            data: None,
            usage: BufferUsage::empty(),
            memory_usage: MemoryUsage::Auto,
            mapped_data: false,
        }
    }
}

impl<'a, T: Copy> Build for BufferBuilder<'a, T> {
    type Target = Buffer<T>;

    fn build(&self) -> Self::Target {
        let count = match self.data {
            Some(data) => (data.len() as vk::DeviceSize).max(self.count.get()),
            None => self.count.get(),
        };

        let buffer_info = vk::BufferCreateInfo::default()
            .size(count * size_of::<T>() as vk::DeviceSize)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .usage(self.usage);

        let flags = if self.mapped_data {
            vk_mem::AllocationCreateFlags::HOST_ACCESS_RANDOM |
            vk_mem::AllocationCreateFlags::MAPPED
        } else {
            vk_mem::AllocationCreateFlags::empty()
        };

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: self.memory_usage.as_vma(),
            flags,
            ..Default::default()
        };

        let (buffer, allocation) = unsafe {
            Context::get().allocator().create_buffer_with_alignment(
                &buffer_info,
                &alloc_info,
                align_of::<T>() as vk::DeviceSize,
            )
        }
        .expect("Failed to create buffer");

        let mapped_data = if self.mapped_data {
            let mapped_data_ptr = Context::get()
                .allocator()
                .get_allocation_info(&allocation)
                .mapped_data as *mut T;

            if !mapped_data_ptr.is_null() {
                Some(unsafe { NonNull::new_unchecked(mapped_data_ptr) })
            } else {
                None
            }
        } else {
            None
        };

        if let Some(data) = self.data {
            if let Some(mapped_data) = mapped_data {
                unsafe {
                    copy_nonoverlapping(
                        data.as_ptr(),
                        mapped_data.as_ptr(),
                        count as usize,
                    )
                };
            }
        }

        Buffer {
            handle: buffer,
            allocation,

            count,
            mapped_data,
        }
    }
}
