use std::{
    num::NonZero,
    ptr::{NonNull, copy_nonoverlapping, slice_from_raw_parts, slice_from_raw_parts_mut},
};

use crate::{CommandBuffer, Context, MemoryUsage, Recording, VkHandle};
use ash::vk;
use utils::{AnyRange, Build, Buildable, Span, ToSpan};
use vk_mem::Alloc;

type DeviceSpan = utils::Span<vk::DeviceSize>;
pub type BufferUsage = vk::BufferUsageFlags;

#[macro_export]
macro_rules! copy_ranges {
    ($(($src:expr => $dst:expr)),*) => {
        [$($crate::BufferCopyRange::new($src, $dst)),*]
    };
}

// --------------------- Buffer region traits ---------------------

pub trait BufferRegionLike<T: Copy> where Self: Sized {
    fn buffer(&self) -> vk::Buffer;
    fn span(&self) -> DeviceSpan;
    fn mapped_data_ptr(&self) -> Option<NonNull<T>>;

    #[inline]
    fn offset(&self) -> vk::DeviceSize {
        self.span().offset
    }

    #[inline]
    fn count(&self) -> vk::DeviceSize {
        self.span().count
    }

    #[inline]
    fn size(&self) -> vk::DeviceSize {
        self.count() * size_of::<T>() as vk::DeviceSize
    }

    #[inline]
    fn mapped<'a>(self) -> Option<&'a [T]> where Self: 'a {
        Some(unsafe {
            &*slice_from_raw_parts(
                self.mapped_data_ptr()?.as_ptr().add(self.offset() as usize),
                self.count() as usize,
            )
        })
    }

    fn copy<'a>(self, dst: impl BufferRegionLike<T> + 'a) where Self: 'a {
        crate::CommandBuffer::run_single_use(|recording| {
            recording.copy_buffer(self, dst);
        });
    }

    fn copy_regions<'a>(self, dst: impl BufferRegionLike<T> + 'a, ranges: &[BufferCopyRange]) {
        crate::CommandBuffer::run_single_use(|recording| {
            recording.copy_buffer_regions(self, dst, ranges);
        });
    }
}

pub trait BufferRegionLikeMut<T: Copy>: BufferRegionLike<T> {
    #[inline]
    fn mapped_mut<'a>(self) -> Option<&'a mut [T]> where Self: 'a {
        Some(unsafe {
            &mut *slice_from_raw_parts_mut(
                self.mapped_data_ptr()?.as_ptr().add(self.offset() as usize),
                self.count() as usize,
            )
        })
    }
}

pub trait GetBufferRegion<T: Copy>
where
    Self: Sized,
{
    fn region<'a>(self, span: impl ToSpan<vk::DeviceSize>) -> BufferRegion<'a, T>
    where
        Self: 'a;
}

pub trait GetBufferRegionMut<T: Copy>
where
    Self: Sized,
{
    fn region_mut<'a>(self, span: impl ToSpan<vk::DeviceSize>) -> BufferRegionMut<'a, T>
    where
        Self: 'a;
}

// --------------------- Buffer ---------------------

#[derive(Debug, cvk_macros::VkHandle)]
pub struct Buffer<T: Copy = u8> {
    handle: vk::Buffer,
    allocation: vk_mem::Allocation,

    count: vk::DeviceSize,
    mapped_data: Option<NonNull<T>>,
}

impl<T: Copy> Buffer<T> {
    #[inline]
    pub const fn count(&self) -> vk::DeviceSize {
        self.count
    }

    #[inline]
    pub const fn size(&self) -> vk::DeviceSize {
        self.count * size_of::<T>() as vk::DeviceSize
    }

    #[inline]
    pub fn mapped(&self) -> Option<&[T]> {
        <&Self as BufferRegionLike<T>>::mapped(self)
    }

    #[inline]
    pub fn mapped_mut(&mut self) -> Option<&mut [T]> {
        <&mut Self as BufferRegionLikeMut<T>>::mapped_mut(self)
    }

    pub fn copy<'a>(&'a self, dst: impl BufferRegionLike<T> + 'a) {
        <&Self as BufferRegionLike<T>>::copy(self, dst)
    }

    pub fn copy_regions<'a>(
        &'a self,
        dst: impl BufferRegionLike<T> + 'a,
        ranges: &[BufferCopyRange],
    ) {
        <&Self as BufferRegionLike<T>>::copy_regions(self, dst, ranges)
    }

    pub fn region(&'_ self, span: impl ToSpan<vk::DeviceSize>) -> BufferRegion<'_, T> {
        <&Self as GetBufferRegion<T>>::region(self, span)
    }

    pub fn region_mut(&'_ mut self, span: impl ToSpan<vk::DeviceSize>) -> BufferRegionMut<'_, T> {
        <&mut Self as GetBufferRegionMut<T>>::region_mut(self, span)
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
    type Builder<'a>
        = BufferBuilder<'a, T>
    where
        T: 'a;
}

impl<T: Copy> BufferRegionLike<T> for &Buffer<T> {
    #[inline]
    fn buffer(&self) -> vk::Buffer {
        self.handle
    }

    #[inline]
    fn span(&self) -> DeviceSpan {
        DeviceSpan::new(0, self.count)
    }

    #[inline]
    fn mapped_data_ptr(&self) -> Option<NonNull<T>> {
        self.mapped_data
    }
}

impl<T: Copy> BufferRegionLike<T> for &mut Buffer<T> {
    #[inline]
    fn buffer(&self) -> vk::Buffer {
        self.handle
    }

    #[inline]
    fn span(&self) -> DeviceSpan {
        DeviceSpan::new(0, self.count)
    }

    #[inline]
    fn mapped_data_ptr(&self) -> Option<NonNull<T>> {
        self.mapped_data
    }
}

impl<T: Copy> BufferRegionLikeMut<T> for &mut Buffer<T> {}

impl<'a, T: Copy> GetBufferRegion<T> for &'a Buffer<T> {
    fn region<'b>(self, span: impl ToSpan<vk::DeviceSize>) -> BufferRegion<'b, T>
    where
        'a: 'b,
    {
        BufferRegion {
            buffer: self,
            span: span.to_span(self.span()),
        }
    }
}

impl<'a, T: Copy> GetBufferRegionMut<T> for &'a mut Buffer<T> {
    fn region_mut<'b>(self, span: impl ToSpan<vk::DeviceSize>) -> BufferRegionMut<'b, T>
    where
        'a: 'b,
    {
        BufferRegionMut {
            span: span.to_span(self.span()),
            buffer: self,
        }
    }
}

// --------------------- Buffer region ---------------------

#[derive(Clone, Copy, Debug)]
pub struct BufferRegion<'a, T: Copy> {
    buffer: &'a Buffer<T>,
    span: DeviceSpan,
}

impl<'a, T: Copy> BufferRegion<'a, T> {
    pub fn new(
        buffer_region: impl GetBufferRegion<T> + 'a,
        span: impl ToSpan<vk::DeviceSize>,
    ) -> Self {
        buffer_region.region(span)
    }

    #[inline]
    pub const fn span(&self) -> DeviceSpan {
        self.span
    }

    #[inline]
    pub const fn offset(&self) -> vk::DeviceSize {
        self.span.offset
    }

    #[inline]
    pub const fn count(&self) -> vk::DeviceSize {
        self.span.count
    }

    #[inline]
    pub const fn size(&self) -> vk::DeviceSize {
        self.span.count * size_of::<T>() as vk::DeviceSize
    }

    #[inline]
    pub fn mapped(self) -> Option<&'a [T]> {
        <Self as BufferRegionLike<T>>::mapped(self)
    }

    pub fn copy(self, dst: impl BufferRegionLike<T> + 'a) {
        <Self as BufferRegionLike<T>>::copy(self, dst)
    }

    pub fn copy_regions(self, dst: impl BufferRegionLike<T> + 'a, ranges: &[BufferCopyRange]) {
        <Self as BufferRegionLike<T>>::copy_regions(self, dst, ranges)
    }

    pub fn region(self, span: impl ToSpan<vk::DeviceSize>) -> BufferRegion<'a, T> {
        <Self as GetBufferRegion<T>>::region(self, span)
    }
}

impl<T: Copy> BufferRegionLike<T> for BufferRegion<'_, T> {
    #[inline]
    fn buffer(&self) -> vk::Buffer {
        self.buffer.handle
    }

    #[inline]
    fn span(&self) -> DeviceSpan {
        self.span
    }

    #[inline]
    fn mapped_data_ptr(&self) -> Option<NonNull<T>> {
        self.buffer.mapped_data
    }
}

impl<'a, T: Copy> GetBufferRegion<T> for BufferRegion<'a, T> {
    fn region<'b>(mut self, span: impl ToSpan<vk::DeviceSize>) -> BufferRegion<'b, T>
    where
        'a: 'b,
    {
        self.span = span.to_span(self.span());
        self
    }
}

impl<'a, T: Copy> From<&'a Buffer<T>> for BufferRegion<'a, T> {
    fn from(buffer: &'a Buffer<T>) -> Self {
        BufferRegion {
            buffer,
            span: DeviceSpan::new(0, buffer.count),
        }
    }
}

#[derive(Debug)]
pub struct BufferRegionMut<'a, T: Copy> {
    buffer: &'a mut Buffer<T>,
    span: DeviceSpan,
}

impl<'a, T: Copy> BufferRegionMut<'a, T> {
    pub fn new(
        buffer_region: impl GetBufferRegionMut<T> + 'a,
        span: impl ToSpan<vk::DeviceSize>,
    ) -> Self {
        buffer_region.region_mut(span)
    }

    #[inline]
    pub const fn span(&self) -> DeviceSpan {
        self.span
    }

    #[inline]
    pub const fn offset(&self) -> vk::DeviceSize {
        self.span.offset
    }

    #[inline]
    pub const fn count(&self) -> vk::DeviceSize {
        self.span.count
    }

    #[inline]
    pub const fn size(&self) -> vk::DeviceSize {
        self.span.count * size_of::<T>() as vk::DeviceSize
    }

    #[inline]
    pub fn mapped(self) -> Option<&'a [T]> {
        <Self as BufferRegionLike<T>>::mapped(self)
    }

    pub fn mapped_mut(self) -> Option<&'a mut [T]> {
        <Self as BufferRegionLikeMut<T>>::mapped_mut(self)
    }

    pub fn region(self, span: impl ToSpan<vk::DeviceSize>) -> BufferRegion<'a, T> {
        <Self as GetBufferRegion<T>>::region(self, span)
    }

    pub fn region_mut(self, span: impl ToSpan<vk::DeviceSize>) -> BufferRegionMut<'a, T> {
        <Self as GetBufferRegionMut<T>>::region_mut(self, span)
    }
}

impl<T: Copy> BufferRegionLike<T> for BufferRegionMut<'_, T> {
    #[inline]
    fn buffer(&self) -> vk::Buffer {
        self.buffer.handle
    }

    #[inline]
    fn span(&self) -> DeviceSpan {
        self.span
    }

    #[inline]
    fn mapped_data_ptr(&self) -> Option<NonNull<T>> {
        self.buffer.mapped_data
    }
}

impl<T: Copy> BufferRegionLikeMut<T> for BufferRegionMut<'_, T> {}

impl<'a, T: Copy> GetBufferRegion<T> for BufferRegionMut<'a, T> {
    fn region<'b>(mut self, span: impl ToSpan<vk::DeviceSize>) -> BufferRegion<'b, T>
    where
        'a: 'b,
    {
        self.span = span.to_span(self.span());
        let Self { buffer, span } = self;
        BufferRegion { buffer, span }
    }
}

impl<'a, T: Copy> GetBufferRegionMut<T> for BufferRegionMut<'a, T> {
    fn region_mut<'b>(mut self, span: impl ToSpan<vk::DeviceSize>) -> BufferRegionMut<'b, T>
    where
        'a: 'b,
    {
        self.span = span.to_span(self.span());
        self
    }
}

impl<'a, T: Copy> From<&'a mut Buffer<T>> for BufferRegionMut<'a, T> {
    fn from(buffer: &'a mut Buffer<T>) -> Self {
        BufferRegionMut {
            span: buffer.span(),
            buffer,
        }
    }
}

// --------------------- Buffer builder ---------------------

#[derive(Clone, Debug, utils::Paramters)]
pub struct BufferBuilder<'a, T: Copy = u8> {
    #[no_param]
    count: NonZero<vk::DeviceSize>,
    #[no_param]
    data: Option<&'a [T]>,
    #[flag]
    usage: BufferUsage,
    memory_usage: MemoryUsage,
    mapped_data: bool,
}

impl<'a, T: Copy> BufferBuilder<'a, T> {
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

impl<T: Copy> Default for BufferBuilder<'_, T> {
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
        assert!(!self.usage.is_empty(), "Buffer usage cannot be empty");

        let count = match self.data {
            Some(data) => (data.len() as vk::DeviceSize).max(self.count.get()),
            None => self.count.get(),
        };

        let buffer_info = vk::BufferCreateInfo::default()
            .size(count * size_of::<T>() as vk::DeviceSize)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .usage(self.usage);

        let flags = if self.mapped_data {
            vk_mem::AllocationCreateFlags::HOST_ACCESS_RANDOM
                | vk_mem::AllocationCreateFlags::MAPPED
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

        let buffer = Buffer {
            handle: buffer,
            allocation,

            count,
            mapped_data,
        };

        if let Some(data) = self.data {
            if let Some(mapped_data) = buffer.mapped_data {
                unsafe { copy_nonoverlapping(data.as_ptr(), mapped_data.as_ptr(), count as usize) };
            } else {
                assert!(
                    self.usage.contains(BufferUsage::TRANSFER_DST),
                    "Building buffer with data and unmapped memory needs usage TRANSFER_DST"
                );

                let staging_buffer = Self::default().staging_buffer().data(data).build();
                CommandBuffer::run_single_use(|recording| {
                    recording.copy_buffer(&staging_buffer, &buffer)
                });
            }
        }

        buffer
    }
}

// --------------------- Buffer commands ---------------------

#[derive(Clone, Debug)]
pub struct BufferCopyRange(AnyRange<vk::DeviceSize>, AnyRange<vk::DeviceSize>);

impl BufferCopyRange {
    #[inline]
    pub fn new(
        src_range: impl Into<AnyRange<vk::DeviceSize>>,
        dst_range: impl Into<AnyRange<vk::DeviceSize>>,
    ) -> Self {
        Self(src_range.into(), dst_range.into())
    }

    #[inline]
    pub fn to_vk<T: Copy>(
        &self,
        src_span: Span<vk::DeviceSize>,
        dst_span: Span<vk::DeviceSize>,
    ) -> vk::BufferCopy {
        let src = self.0.clone().to_span(src_span);
        let dst = self.1.clone().to_span(dst_span);

        vk::BufferCopy::default()
            .size(src.count.min(dst.count) * size_of::<T>() as vk::DeviceSize)
            .src_offset(src.offset * size_of::<T>() as vk::DeviceSize)
            .dst_offset(dst.offset * size_of::<T>() as vk::DeviceSize)
    }
}

impl<T: Into<AnyRange<vk::DeviceSize>>, U: Into<AnyRange<vk::DeviceSize>>> From<(T, U)>
    for BufferCopyRange
{
    fn from((src, dst): (T, U)) -> Self {
        Self::new(src, dst)
    }
}

impl<'a> Recording<'a> {
    pub fn copy_buffer<T: Copy>(
        &mut self,
        src_region: impl BufferRegionLike<T> + 'a,
        dst_region: impl BufferRegionLike<T> + 'a,
    ) {
        let DeviceSpan {
            offset: src_offset,
            count: src_count,
        } = src_region.span();

        let DeviceSpan {
            offset: dst_offset,
            count: dst_count,
        } = dst_region.span();

        let size = src_count.min(dst_count) * size_of::<T>() as vk::DeviceSize;

        let raw_region = vk::BufferCopy::default()
            .size(size)
            .src_offset(src_offset * size_of::<T>() as vk::DeviceSize)
            .dst_offset(dst_offset * size_of::<T>() as vk::DeviceSize);

        unsafe {
            Context::get_device().cmd_copy_buffer(
                self.handle(),
                src_region.buffer(),
                dst_region.buffer(),
                &[raw_region],
            );
        }
    }

    pub fn copy_buffer_regions<T: Copy>(
        &mut self,
        src_region: impl BufferRegionLike<T> + 'a,
        dst_region: impl BufferRegionLike<T> + 'a,
        ranges: &[BufferCopyRange],
    ) {
        let raw_regions: Vec<_> = ranges
            .iter()
            .map(|copy_range| copy_range.to_vk::<T>(src_region.span(), dst_region.span()))
            .collect();

        unsafe {
            Context::get_device().cmd_copy_buffer(
                self.handle(),
                src_region.buffer(),
                dst_region.buffer(),
                &raw_regions,
            );
        }
    }
}
