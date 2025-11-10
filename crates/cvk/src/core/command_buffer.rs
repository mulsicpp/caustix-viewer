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

    pub fn run_single_use(recorder: impl FnOnce(&Recording)) {
        let recording = Self::new(CommandBufferUses::Single).start_recording();

        recorder(&recording);

        recording.submit().wait();
    }

    pub fn start_recording(self) -> Recording {
        assert!(self.usable, "Command buffer is no longer usable");

        let flags = match self.uses {
            CommandBufferUses::Single => vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            CommandBufferUses::Multi => vk::CommandBufferUsageFlags::empty(),
        };

        let info = vk::CommandBufferBeginInfo::default().flags(flags);

        self.fence.wait();
        unsafe { Context::get_device().begin_command_buffer(self.handle, &info) }
            .expect("Failed to start recording of command buffer");

        Recording(self)
    }

    pub fn wait(&self) {
        self.fence.wait();
    }

    pub fn wait_with_timeout(&self, timeout: u64) {
        self.fence.wait_with_timeout(timeout);
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        println!("dropping cmd buf");

        self.wait();
        unsafe {
            Context::get_device()
                .free_command_buffers(Context::get().device().command_pool, &[self.handle]);
        }
    }
}

pub struct Recording(CommandBuffer);

impl Recording {
    pub fn submit(mut self) -> CommandBuffer {
        unsafe { Context::get_device().end_command_buffer(self.0.handle) }
            .expect("Failed to end recording of command buffer");

        let handles = [self.handle()];

        let submit_info = vk::SubmitInfo::default().command_buffers(handles.as_slice());

        if self.0.uses == CommandBufferUses::Single {
            self.0.usable = false;
        }
        self.0.fence.reset();

        unsafe { Context::get_device().queue_submit(Context::get().device().main_queue.handle(), &[submit_info], self.0.fence.handle()) }
            .expect("Failed to submit command buffer");

        self.0
    }
}

impl VkHandle for Recording {
    type HandleType = vk::CommandBuffer;

    fn handle(&self) -> Self::HandleType {
        self.0.handle()
    }
}
