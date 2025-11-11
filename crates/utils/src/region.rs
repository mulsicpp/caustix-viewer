use std::ops::{Add, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive, Sub};

pub trait RegionPrimitive: Copy + Add<Self, Output = Self> + Sub<Self, Output = Self> {
    const ZERO: Self;
    const ONE: Self;

    fn saturating_sub(self, rhs: Self) -> Self;
}

macro_rules! impl_region_primitive {
    ($prim:ty) => {
        impl RegionPrimitive for $prim {
            const ZERO: Self = 0;

            const ONE: Self = 1;

            fn saturating_sub(self, rhs: Self) -> Self {
                self.saturating_sub(rhs)
            }
        }
    };
}

impl_region_primitive!{u8}
impl_region_primitive!{u16}
impl_region_primitive!{u32}
impl_region_primitive!{u64}
impl_region_primitive!{usize}

impl_region_primitive!{i8}
impl_region_primitive!{i16}
impl_region_primitive!{i32}
impl_region_primitive!{i64}
impl_region_primitive!{isize}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region<T> {
    pub offset: T,
    pub count: T,
}

impl<T> Region<T> {
    pub fn new(offset: T, count: T) -> Self {
        Self { offset, count }
    }
}

pub trait ToRegion<T>
where
    T: RegionPrimitive,
{
    fn to_region(self, count: T) -> Region<T>;
}

impl<T> ToRegion<T> for T
where
    T: RegionPrimitive,
{
    fn to_region(self, _: T) -> Region<T> {
        Region {
            offset: self,
            count: T::ONE,
        }
    }
}

impl<T> ToRegion<T> for Range<T>
where
    T: RegionPrimitive,
{
    fn to_region(self, _: T) -> Region<T> {
        Region {
            offset: self.start,
            count: self.end - self.start,
        }
    }
}

impl<T> ToRegion<T> for RangeInclusive<T>
where
    T: RegionPrimitive,
{
    fn to_region(self, _: T) -> Region<T> {
        Region {
            offset: *self.start(),
            count: *self.end() - *self.start() + T::ONE,
        }
    }
}

impl<T> ToRegion<T> for RangeTo<T>
where
    T: RegionPrimitive,
{
    fn to_region(self, _: T) -> Region<T> {
        Region {
            offset: T::ZERO,
            count: self.end,
        }
    }
}

impl<T> ToRegion<T> for RangeToInclusive<T>
where
    T: RegionPrimitive,
{
    fn to_region(self, _: T) -> Region<T> {
        Region {
            offset: T::ZERO,
            count: self.end + T::ONE,
        }
    }
}

impl<T> ToRegion<T> for RangeFrom<T>
where
    T: RegionPrimitive,
{
    fn to_region(self, count: T) -> Region<T> {
        Region {
            offset: self.start,
            count: count.saturating_sub(self.start),
        }
    }
}

impl<T> ToRegion<T> for RangeFull
where
    T: RegionPrimitive,
{
    fn to_region(self, count: T) -> Region<T> {
        Region::<T> {
            offset: T::ZERO,
            count: count,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AnyRange<T: RegionPrimitive> {
    Value(T),
    Range(Range<T>),
    RangeInclusive(RangeInclusive<T>),
    RangeTo(RangeTo<T>),
    RangeToInclusive(RangeToInclusive<T>),
    RangeFrom(RangeFrom<T>),
    RangeFull(RangeFull),
}

impl<T: RegionPrimitive> ToRegion<T> for AnyRange<T> {
    fn to_region(self, count: T) -> Region<T> {
        match self {
            AnyRange::Value(value) => value.to_region(count),
            AnyRange::Range(range) => range.to_region(count),
            AnyRange::RangeInclusive(range_inclusive) => range_inclusive.to_region(count),
            AnyRange::RangeTo(range_to) => range_to.to_region(count),
            AnyRange::RangeToInclusive(range_to_inclusive) => range_to_inclusive.to_region(count),
            AnyRange::RangeFrom(range_from) => range_from.to_region(count),
            AnyRange::RangeFull(range_full) => range_full.to_region(count),
        }
    }
}

impl<T: RegionPrimitive> From<T> for AnyRange<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T: RegionPrimitive> From<Range<T>> for AnyRange<T> {
    fn from(value: Range<T>) -> Self {
        Self::Range(value)
    }
}

impl<T: RegionPrimitive> From<RangeInclusive<T>> for AnyRange<T> {
    fn from(value: RangeInclusive<T>) -> Self {
        Self::RangeInclusive(value)
    }
}

impl<T: RegionPrimitive> From<RangeTo<T>> for AnyRange<T> {
    fn from(value: RangeTo<T>) -> Self {
        Self::RangeTo(value)
    }
}

impl<T: RegionPrimitive> From<RangeToInclusive<T>> for AnyRange<T> {
    fn from(value: RangeToInclusive<T>) -> Self {
        Self::RangeToInclusive(value)
    }
}

impl<T: RegionPrimitive> From<RangeFrom<T>> for AnyRange<T> {
    fn from(value: RangeFrom<T>) -> Self {
        Self::RangeFrom(value)
    }
}

impl<T: RegionPrimitive> From<RangeFull> for AnyRange<T> {
    fn from(value: RangeFull) -> Self {
        Self::RangeFull(value)
    }
}