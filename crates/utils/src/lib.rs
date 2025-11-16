
pub mod build;
pub mod ptr;
pub mod span;

pub use build::*;
pub use ptr::*;
pub use span::*;

pub use util_macros::Paramters;
pub use util_macros::Share;

pub use std::sync::Arc as Shared;

pub trait Share {
    type Internal;

    fn share(self) -> Shared<Self::Internal>;
}

impl<T> Share for &Shared<T> {
    type Internal = T;

    #[inline]
    fn share(self) -> Shared<Self::Internal> {
        self.clone()
    }
}

#[cfg(test)]
pub mod tests;