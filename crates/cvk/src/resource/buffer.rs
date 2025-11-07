use std::{
    num::NonZero,
    ptr::{NonNull, slice_from_raw_parts, slice_from_raw_parts_mut},
};

use ash::vk;
use vk_mem::Alloc;

use crate::{Context, MemoryUsage};

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
    type Builder = BufferBuilder;
}

#[derive(Clone, utils::Paramters)]
pub struct BufferBuilder {
    #[no_param]
    size: NonZero<vk::DeviceSize>,
    #[no_param]
    align: vk::DeviceSize,
    #[no_param]
    data: Option<NonNull<u8>>,
    usage: BufferUsage,
    memory_usage: MemoryUsage,
    mapped_data: bool,
}

impl BufferBuilder {
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

    pub fn data_slice<T>(mut self, slice: &[T]) -> Self {
        self = self.count_of::<T>(slice.len() as vk::DeviceSize);
        self.data = slice
            .first()
            .and_then(|first| NonNull::new(first as *const T as *mut T as *mut u8));
        self
    }

    pub fn data<T>(mut self, value: &T) -> Self {
        self = self.count_of::<T>(1 as vk::DeviceSize);
        self.data = Some(unsafe { NonNull::new_unchecked(value as *const T as *mut T as *mut u8) });
        self
    }

    pub fn staging_buffer(self) -> Self {
        self.usage(BufferUsage::TRANSFER_SRC)
            .memory_usage(MemoryUsage::PreferHost)
            .mapped_data(true)
    }
}

impl Default for BufferBuilder {
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

impl Build for BufferBuilder {
    type Target = Buffer;

    fn build(&self) -> Self::Target {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(self.size.get())
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .usage(self.usage);

        let required_flags = if self.mapped_data {
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE
        } else {
            vk::MemoryPropertyFlags::empty()
        };

        let memory_info = vk_mem::AllocationCreateInfo {
            usage: self.memory_usage.as_vma(),
            required_flags,
            ..Default::default()
        };

        let (buffer, mut allocation) = unsafe {
            Context::get().allocator().create_buffer_with_alignment(
                &buffer_info,
                &memory_info,
                self.align,
            )
        }
        .expect("Failed to create buffer");

        let mapped_data = if self.mapped_data {
            let mapped_data_ptr =
                unsafe { Context::get().allocator().map_memory(&mut allocation) }.unwrap();

            if !mapped_data_ptr.is_null() {
                Some(unsafe { NonNull::new_unchecked(mapped_data_ptr) })
            } else {
                None
            }
        } else {
            None
        };

        Buffer {
            handle: buffer,
            allocation,

            size: self.size.get(),
            align: self.align,
            mapped_data,
        }
    }
}
