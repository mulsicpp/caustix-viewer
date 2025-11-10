use std::{
    num::NonZero,
    ptr::{NonNull, copy_nonoverlapping, slice_from_raw_parts, slice_from_raw_parts_mut},
};

use ash::vk;
use vk_mem::Alloc;

use crate::{Context, MemoryUsage, Recording, VkHandle};

use utils::{Build, Buildable};

pub type BufferUsage = vk::BufferUsageFlags;

#[derive(cvk_macros::VkHandle)]
pub struct Buffer {
    handle: vk::Buffer,
    allocation: vk_mem::Allocation,

    size: vk::DeviceSize,
    align: vk::DeviceSize,
    mapped_data: Option<NonNull<u8>>,
}

impl Buffer {
    pub fn size(&self) -> vk::DeviceSize {
        self.size
    }

    pub fn align(&self) -> vk::DeviceSize {
        self.align
    }

    pub fn mapped<T>(&self) -> Option<&T> {
        if size_of::<T>() <= self.size as usize {
            unsafe { Some(&*(self.mapped_data?.as_ptr() as *mut T)) }
        } else {
            None
        }
    }

    pub fn mapped_mut<T>(&mut self) -> Option<&mut T> {
        if size_of::<T>() <= self.size as usize {
            unsafe { Some(&mut *(self.mapped_data?.as_ptr() as *mut T)) }
        } else {
            None
        }
    }

    pub fn mapped_slice<T>(&self) -> Option<&[T]> {
        Some(unsafe {
            &*slice_from_raw_parts(
                self.mapped_data?.as_ptr() as *const T,
                self.size as usize / size_of::<T>(),
            )
        })
    }

    pub fn mapped_slice_mut<T>(&mut self) -> Option<&mut [T]> {
        Some(unsafe {
            &mut *slice_from_raw_parts_mut(
                self.mapped_data?.as_ptr() as *mut T,
                self.size as usize / size_of::<T>(),
            )
        })
    }
}

#[derive(utils::Paramters, Default)]
pub struct BufferCopyRegion {
    pub src_offset: vk::DeviceSize,
    pub dst_offset: vk::DeviceSize,
    pub size: vk::DeviceSize,
}

impl BufferCopyRegion {
    fn as_raw(&self) -> vk::BufferCopy {
        let Self {
            src_offset,
            dst_offset,
            size,
        } = *self;
        vk::BufferCopy {
            src_offset,
            dst_offset,
            size,
        }
    }
}

impl Recording {
    pub fn copy_buffer(&self, src: &Buffer, dst: &mut Buffer) {
        let raw_region = vk::BufferCopy::default().size(src.size.min(dst.size));
        unsafe {
            Context::get_device().cmd_copy_buffer(
                self.handle(),
                src.handle,
                dst.handle,
                &[raw_region],
            );
        }
    }

    pub fn copy_buffer_regions(
        &self,
        src: &Buffer,
        dst: &mut Buffer,
        copy_regions: &[BufferCopyRegion],
    ) {
        let raw_regions: Vec<_> = copy_regions.iter().map(|region| region.as_raw()).collect();
        unsafe {
            Context::get_device().cmd_copy_buffer(
                self.handle(),
                src.handle,
                dst.handle,
                &raw_regions,
            );
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            Context::get()
                .allocator()
                .destroy_buffer(self.handle, &mut self.allocation);
        }
    }
}

impl Buildable for Buffer {
    type Builder<'a> = BufferBuilder<'a>;
}

#[derive(Clone, Debug, utils::Paramters)]
pub struct BufferBuilder<'a> {
    #[no_param]
    size: NonZero<vk::DeviceSize>,
    #[no_param]
    align: vk::DeviceSize,
    #[no_param]
    data: Option<utils::ScopedPtr<'a, u8>>,
    #[flag]
    usage: BufferUsage,
    memory_usage: MemoryUsage,
    mapped_data: bool,
}

impl<'a> BufferBuilder<'a> {
    pub fn size(mut self, size: impl Into<vk::DeviceSize>) -> Self {
        self.size = NonZero::new(size.into()).expect("Buffer size needs to be greater than zero");
        self.align = 1;
        self.data = None;
        self
    }

    pub fn size_aligned(
        mut self,
        size: impl Into<vk::DeviceSize>,
        align: impl Into<vk::DeviceSize>,
    ) -> Self {
        self.size = NonZero::new(size.into()).expect("Buffer size needs to be greater than zero");
        self.align = align.into();
        self.data = None;
        self
    }

    pub fn count_of<T>(self, count: impl Into<vk::DeviceSize>) -> Self {
        self.size_aligned(
            count.into() * size_of::<T>() as vk::DeviceSize,
            align_of::<T>() as vk::DeviceSize,
        )
    }

    pub fn data_slice<T>(mut self, slice: &'a [T]) -> Self {
        self = self.count_of::<T>(slice.len() as vk::DeviceSize);
        self.data = slice
            .first()
            .and_then(|first| utils::ScopedPtr::new(first as *const T as *mut T as *mut u8));

        self
    }

    pub fn data<T>(mut self, value: &T) -> Self {
        self = self.count_of::<T>(1 as vk::DeviceSize);
        self.data = Some(unsafe {
            utils::ScopedPtr::new_unchecked(value as *const T as *mut T as *mut u8)
        });
        self
    }

    pub fn staging_buffer(self) -> Self {
        self.usage(BufferUsage::TRANSFER_SRC)
            .memory_usage(MemoryUsage::PreferHost)
            .mapped_data(true)
    }
}

impl Default for BufferBuilder<'_> {
    fn default() -> Self {
        Self {
            align: 1,
            size: unsafe { NonZero::new_unchecked(1) },
            data: None,
            usage: BufferUsage::empty(),
            memory_usage: MemoryUsage::Auto,
            mapped_data: false,
        }
    }
}

impl<'a> Build for BufferBuilder<'a> {
    type Target = Buffer;

    fn build(&self) -> Self::Target {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(self.size.get())
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .usage(self.usage);

        let flags = if self.mapped_data {
            vk_mem::AllocationCreateFlags::HOST_ACCESS_RANDOM |
            //vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE |
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
                self.align,
            )
        }
        .expect("Failed to create buffer");

        let mapped_data = if self.mapped_data {
            let mapped_data_ptr = Context::get()
                .allocator()
                .get_allocation_info(&allocation)
                .mapped_data as *mut u8;

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
                        data.as_ptr() as *const u8,
                        mapped_data.as_ptr(),
                        self.size.get() as usize,
                    )
                };
            }
        }

        Buffer {
            handle: buffer,
            allocation,

            size: self.size.get(),
            align: self.align,
            mapped_data,
        }
    }
}
