use std::marker::PhantomData;

use ash::vk;

use crate::{Context, Fence, VkHandle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandBufferUses {
    Single,
    Multi,
}

#[derive(cvk_macros::VkHandle)]
pub struct CommandBuffer {
    handle: vk::CommandBuffer,
    fence: Fence,
    uses: CommandBufferUses,
    usable: bool,
}

impl CommandBuffer {
    pub fn new(uses: CommandBufferUses) -> Self {
        let info = vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(1u32)
            .command_pool(Context::get().device().command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let handle = unsafe { Context::get_device().allocate_command_buffers(&info) }
            .expect("Failed to allocate command buffer")[0];

        let fence = Fence::new(true);

        Self {
            handle,
            fence,
            uses,
            usable: true,
        }
    }

    pub fn run_single_use<'a>(recorder: impl FnOnce(&mut Recording<'a>)) {
        let mut recording = Self::new(CommandBufferUses::Single).start_recording();

        recorder(&mut recording);

        recording.submit().wait();
    }

    pub fn start_recording<'a>(self) -> Recording<'a> {
        assert!(self.usable, "Command buffer is no longer usable");

        let flags = match self.uses {
            CommandBufferUses::Single => vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            CommandBufferUses::Multi => vk::CommandBufferUsageFlags::empty(),
        };

        let info = vk::CommandBufferBeginInfo::default().flags(flags);

        self.fence.wait();
        unsafe { Context::get_device().begin_command_buffer(self.handle, &info) }
            .expect("Failed to start recording of command buffer");

        Recording { cmd_buf: self, _marker: PhantomData::default() }
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        println!("dropping cmd buf");

        self.fence.wait();
        unsafe {
            Context::get_device()
                .free_command_buffers(Context::get().device().command_pool, &[self.handle]);
        }
    }
}

pub struct Recording<'a> {
    cmd_buf: CommandBuffer,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Recording<'a> {
    pub fn submit(mut self) -> SubmittedRecording<'a> {
        unsafe { Context::get_device().end_command_buffer(self.cmd_buf.handle) }
            .expect("Failed to end recording of command buffer");

        let handles = [self.handle()];

        let submit_info = vk::SubmitInfo::default().command_buffers(handles.as_slice());

        if self.cmd_buf.uses == CommandBufferUses::Single {
            self.cmd_buf.usable = false;
        }
        self.cmd_buf.fence.reset();

        unsafe { Context::get_device().queue_submit(Context::get().device().main_queue.handle(), &[submit_info], self.cmd_buf.fence.handle()) }
            .expect("Failed to submit command buffer");

        SubmittedRecording { cmd_buf: self.cmd_buf, _marker: self._marker }
    }
}

impl<'a> VkHandle for Recording<'a> {
    type HandleType = vk::CommandBuffer;

    fn handle(&self) -> Self::HandleType {
        self.cmd_buf.handle()
    }
}

pub struct SubmittedRecording<'a> {
    cmd_buf: CommandBuffer,
    _marker: PhantomData<&'a ()>,
}


impl<'a> SubmittedRecording<'a> {
    pub fn wait(self) -> CommandBuffer {
        self.cmd_buf.fence.wait();
        self.cmd_buf
    }
}