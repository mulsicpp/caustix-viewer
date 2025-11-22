use std::usize;

use ash::vk;
use winit::dpi::PhysicalSize;

use crate::{
    Extent2D, Format, Image, ImageUsage,
    core::{device::Device, instance::Surface},
};

pub type PresentMode = vk::PresentModeKHR;
pub type SurfaceFormat = vk::SurfaceFormatKHR;
pub type ColorSpace = vk::ColorSpaceKHR;

#[derive(cvk_macros::VkHandle, Debug)]
pub struct Swapchain {
    handle: vk::SwapchainKHR,
    images: Vec<utils::Shared<Image>>,

    surface_format: SurfaceFormat,
    present_mode: PresentMode,
    extent: Extent2D,
    image_usage: ImageUsage,
}

impl Swapchain {
    pub fn new(device: &Device, surface: &Surface, info: &SwapchainInfo) -> Self {
        let surface_format = Self::get_surface_format(
            device.physical_device,
            surface,
            info.format_preference.as_slice(),
        );
        let present_mode =
            Self::get_present_mode(device.physical_device, surface, info.desired_present_mode);

        let mut swapchain = Self {
            handle: vk::SwapchainKHR::null(),
            images: vec![],
            surface_format,
            present_mode,
            extent: (0, 0).into(),
            image_usage: info.image_usage,
        };

        swapchain.recreate(device, surface);

        swapchain
    }

    pub fn recreate(&mut self, device: &Device, surface: &Surface) {
        let swapchain_fns =
            device.extensions.swapchain.as_ref().expect(
                "Failed to create swapchain, because the required extension was not enabled",
            );

        let capabilities = unsafe {
            surface
                .fns
                .get_physical_device_surface_capabilities(device.physical_device, surface.handle)
        }
        .expect("Failed to aquire surface capabilities");

        assert!(!self.image_usage.is_empty(), "Image usage cannot be empty");
        assert!(
            self.images
                .iter()
                .all(|image| utils::Shared::strong_count(image) < 2),
            "Swapchain images are still in use"
        );

        let extent: Extent2D;
        if capabilities.current_extent.width != u32::MAX {
            let vk::Extent2D { width, height } = capabilities.current_extent;
            extent = Extent2D { width, height }
        } else {
            let PhysicalSize::<u32> { width, height } = surface.window.inner_size();

            extent = Extent2D {
                width: u32::clamp(
                    width,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: u32::clamp(
                    height,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            };
        }
        self.extent = extent;

        let image_count =
            if (1..(capabilities.min_image_count + 1)).contains(&capabilities.max_image_count) {
                capabilities.max_image_count
            } else {
                capabilities.min_image_count + 1
            };

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .old_swapchain(self.handle)
            .surface(surface.handle)
            .present_mode(self.present_mode)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .clipped(true)
            .pre_transform(capabilities.current_transform)
            .min_image_count(image_count)
            .image_format(self.surface_format.format)
            .image_color_space(self.surface_format.color_space)
            .image_extent(self.extent.to_vk())
            .image_usage(self.image_usage)
            .image_array_layers(1);

        let family_indices = [
            device.main_queue.family_idx,
            device.present_queue.family_idx,
        ];

        let create_info = if family_indices[0] != family_indices[1] {
            create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&family_indices)
        } else {
            create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        };
        unsafe {
            self.handle = swapchain_fns
                .create_swapchain(&create_info, None)
                .expect("Failed to create swapchain");
            self.images = swapchain_fns
                .get_swapchain_images(self.handle)
                .expect("Failed to get swapchain images").into_iter().map(|image| Image::swapchain_image(self, image).share()).collect();
        }
    }

    #[inline]
    pub const fn format(&self) -> SurfaceFormat {
        self.surface_format
    }

    #[inline]
    pub const fn extent(&self) -> Extent2D {
        self.extent
    }

    #[inline]
    pub const fn image_usage(&self) -> ImageUsage {
        self.image_usage
    }

    #[inline]
    pub const fn present_mode(&self) -> PresentMode {
        self.present_mode
    }

    fn get_surface_format(
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
        format_preference: &[Format],
    ) -> SurfaceFormat {
        let surface_formats = unsafe {
            surface
                .fns
                .get_physical_device_surface_formats(physical_device, surface.handle)
        }
        .expect("Failed to aquire surface formats");

        let mut chosen_format = None;
        for surface_format in surface_formats {
            let i = format_preference
                .iter()
                .position(|format| *format == surface_format.format)
                .unwrap_or(format_preference.len());
            if i < chosen_format.map(|(_, i)| i).unwrap_or(usize::MAX) {
                chosen_format = Some((surface_format, i));
            }
        }

        chosen_format
            .expect("No suitable surface format for swapchain found")
            .0
    }

    fn get_present_mode(
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
        present_mode: PresentMode,
    ) -> PresentMode {
        let present_modes = unsafe {
            surface
                .fns
                .get_physical_device_surface_present_modes(physical_device, surface.handle)
        }
        .expect("Failed to aquire present modes");

        if present_modes.contains(&present_mode) {
            present_mode
        } else {
            PresentMode::FIFO
        }
    }
}

#[derive(utils::Paramters, Debug, Clone)]
pub struct SwapchainInfo {
    #[vec(push_preferred_format)]
    format_preference: Vec<Format>,
    desired_present_mode: PresentMode,
    #[flag]
    image_usage: ImageUsage,
}

impl Default for SwapchainInfo {
    fn default() -> Self {
        Self {
            format_preference: vec![],
            desired_present_mode: PresentMode::MAILBOX,
            image_usage: ImageUsage::empty(),
        }
    }
}
