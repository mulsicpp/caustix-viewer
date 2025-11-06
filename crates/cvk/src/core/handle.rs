

pub trait VkHandle {
    type HandleType;

    fn handle(&self) -> Self::HandleType;
}