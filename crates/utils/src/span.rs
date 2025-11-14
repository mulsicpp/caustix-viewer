use std::ops::{Add, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive, Sub};

pub trait SpanPrimitive:
    Copy + Add<Self, Output = Self> + Sub<Self, Output = Self> + PartialOrd
{
    const ZERO: Self;
    const ONE: Self;

    fn saturating_sub(self, rhs: Self) -> Self;
}

macro_rules! impl_span_primitive {
    ($prim:ty) => {
        impl SpanPrimitive for $prim {
            const ZERO: Self = 0;

            const ONE: Self = 1;

            fn saturating_sub(self, rhs: Self) -> Self {
                self.saturating_sub(rhs)
            }
        }
    };
}

impl_span_primitive! {u8}
impl_span_primitive! {u16}
impl_span_primitive! {u32}
impl_span_primitive! {u64}
impl_span_primitive! {usize}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span<T: SpanPrimitive> {
    pub offset: T,
    pub count: T,
}

impl<T: SpanPrimitive> Span<T> {
    pub fn new(offset: T, count: T) -> Self {
        Self { offset, count }
    }

    pub fn invalid() -> Self {
        Self { offset: T::ZERO, count: T::ZERO }
    }
}

pub trait ToSpan<T>
where
    T: SpanPrimitive,
{
    fn to_span(self, span: Span<T>) -> Span<T>;
}

impl<T> ToSpan<T> for Span<T> where T: SpanPrimitive {
    fn to_span(self, span: Span<T>) -> Span<T> {
        if self.offset + self.count <= span.count {
            Span::new(span.offset + self.offset, self.count)
        } else {
            Span::invalid()
        }
    }
}

impl<T> ToSpan<T> for T
where
    T: SpanPrimitive,
{
    fn to_span(self, span: Span<T>) -> Span<T> {
        if self < span.count {
            Span::new(span.offset + self, T::ONE)
        } else {
            Span::invalid()
        }
    }
}

impl<T> ToSpan<T> for Range<T>
where
    T: SpanPrimitive,
{
    fn to_span(self, span: Span<T>) -> Span<T> {
        if self.end <= span.count {
            Span::new(span.offset + self.start, self.end.saturating_sub(self.start))
        } else {
            Span::invalid()
        }
    }
}

impl<T> ToSpan<T> for RangeInclusive<T>
where
    T: SpanPrimitive,
{
    fn to_span(self, span: Span<T>) -> Span<T> {
        if *self.end() < span.count {
            Span::new(span.offset + *self.start(), self.end().saturating_sub(*self.start()) + T::ONE)
        } else {
            Span::invalid()
        }
    }
}

impl<T> ToSpan<T> for RangeTo<T>
where
    T: SpanPrimitive,
{
    fn to_span(self, span: Span<T>) -> Span<T> {
        if self.end <= span.count {
            Span::new(span.offset, self.end)
        } else {
            Span::invalid()
        }
    }
}

impl<T> ToSpan<T> for RangeToInclusive<T>
where
    T: SpanPrimitive,
{
    fn to_span(self, span: Span<T>) -> Span<T> {
        if self.end < span.count {
            Span::new(span.offset, self.end + T::ONE)
        } else {
            Span::invalid()
        }
    }
}

impl<T> ToSpan<T> for RangeFrom<T>
where
    T: SpanPrimitive,
{
    fn to_span(self, span: Span<T>) -> Span<T> {
        if self.start < span.count {
            Span::new(span.offset + self.start, span.count - self.start)
        } else {
            Span::invalid()
        }
    }
}

impl<T> ToSpan<T> for RangeFull
where
    T: SpanPrimitive,
{
    fn to_span(self, span: Span<T>) -> Span<T> {
        span
    }
}

#[derive(Clone, Debug)]
pub enum AnyRange<T: SpanPrimitive> {
    Value(T),
    Range(Range<T>),
    RangeInclusive(RangeInclusive<T>),
    RangeTo(RangeTo<T>),
    RangeToInclusive(RangeToInclusive<T>),
    RangeFrom(RangeFrom<T>),
    RangeFull(RangeFull),
}

impl<T: SpanPrimitive> ToSpan<T> for AnyRange<T> {
    fn to_span(self, span: Span<T>) -> Span<T> {
        match self {
            AnyRange::Value(value) => value.to_span(span),
            AnyRange::Range(range) => range.to_span(span),
            AnyRange::RangeInclusive(range_inclusive) => range_inclusive.to_span(span),
            AnyRange::RangeTo(range_to) => range_to.to_span(span),
            AnyRange::RangeToInclusive(range_to_inclusive) => range_to_inclusive.to_span(span),
            AnyRange::RangeFrom(range_from) => range_from.to_span(span),
            AnyRange::RangeFull(range_full) => range_full.to_span(span),
        }
    }
}

impl<T: SpanPrimitive> From<T> for AnyRange<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T: SpanPrimitive> From<Range<T>> for AnyRange<T> {
    fn from(value: Range<T>) -> Self {
        Self::Range(value)
    }
}

impl<T: SpanPrimitive> From<RangeInclusive<T>> for AnyRange<T> {
    fn from(value: RangeInclusive<T>) -> Self {
        Self::RangeInclusive(value)
    }
}

impl<T: SpanPrimitive> From<RangeTo<T>> for AnyRange<T> {
    fn from(value: RangeTo<T>) -> Self {
        Self::RangeTo(value)
    }
}

impl<T: SpanPrimitive> From<RangeToInclusive<T>> for AnyRange<T> {
    fn from(value: RangeToInclusive<T>) -> Self {
        Self::RangeToInclusive(value)
    }
}

impl<T: SpanPrimitive> From<RangeFrom<T>> for AnyRange<T> {
    fn from(value: RangeFrom<T>) -> Self {
        Self::RangeFrom(value)
    }
}

impl<T: SpanPrimitive> From<RangeFull> for AnyRange<T> {
    fn from(value: RangeFull) -> Self {
        Self::RangeFull(value)
    }
}
