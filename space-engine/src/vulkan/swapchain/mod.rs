mod surface;
use surface::EngineSurface;

use anyhow::Result;

use winit::dpi::PhysicalSize;
use winit::window::Window;

use std::sync::Arc;

use ash::vk;

use crate::utils::image_utils::create_image_view;
use crate::vulkan::VulkanContext;

pub struct EngineSwapchainImage {
    pub handle: vk::Image,
    pub view: vk::ImageView,
    pub command_buffer: vk::CommandBuffer,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
}

pub struct EngineSwapchain {
    pub desired_image_count: u32,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub images: Vec<EngineSwapchainImage>,
    pub command_pool: vk::CommandPool,
    handle: vk::SwapchainKHR,
    surface: EngineSurface,
    window: Arc<Window>,
    context: Arc<VulkanContext>,
    pub is_dirty: bool,
}

impl EngineSwapchain {
    pub fn new(context: Arc<VulkanContext>, window: Arc<Window>) -> Result<Self> {
        let surface = EngineSurface::new(
            &context.entry,
            &context.instance,
            &context.phys_device.handle,
            &context.surface_loader,
            &window,
        )?;

        let format = vk::Format::B8G8R8A8_SRGB;
        let extent = if surface.capabilities.current_extent.width != u32::MAX {
            surface.capabilities.current_extent
        } else {
            let window_size = window.inner_size();
            vk::Extent2D {
                width: window_size.width,
                height: window_size.height,
            }
        };
        let desired_image_count = (surface.capabilities.min_image_count + 1).clamp(
            surface.capabilities.min_image_count,
            if surface.capabilities.max_image_count == 0 {
                u32::MAX
            } else {
                surface.capabilities.max_image_count
            },
        );

        let command_pool = unsafe {
            context.device.handle.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(context.device.qfs.graphics.index)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )
        }?;

        Ok(Self {
            desired_image_count,
            format,
            extent,
            images: Default::default(),
            handle: Default::default(),
            surface,
            window,
            context,
            is_dirty: true,
            command_pool,
        })
    }

    // Create the new swapchain on resize and destroy the old one.
    pub fn resize(&mut self, size: PhysicalSize<u32>) -> Result<()> {
        self.extent = vk::Extent2D {
            width: size.width,
            height: size.height,
        };

        if self.extent.width == 0 || self.extent.height == 0 {
            return Ok(());
        }

        self.is_dirty = false;

        // Create info for the new swapchain.
        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(self.surface.handle)
            .min_image_count(self.desired_image_count)
            .image_format(self.format)
            .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .image_extent(self.extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::MAILBOX)
            .clipped(true)
            .old_swapchain(self.handle);

        let new_swapchain = unsafe {
            self.context
                .swapchain_loader
                .create_swapchain(&create_info, None)
                .expect("Failed to create swapchain")
        };
        // Clearing the old swapchain content and itself.
        self.images.iter().for_each(|image| {
            unsafe {
                self.context
                    .device
                    .handle
                    .destroy_image_view(image.view, None)
            };
        });

        unsafe {
            self.context
                .swapchain_loader
                .destroy_swapchain(self.handle, None)
        };

        // Creating a new swapchain and its image views.
        self.handle = new_swapchain;

        let new_images = unsafe {
            self.context
                .swapchain_loader
                .get_swapchain_images(self.handle)?
        };

        let new_images_len = new_images.len();

        if self.images.len() == new_images_len {
            self.images
                .iter_mut()
                .enumerate()
                .for_each(|(i, engine_image)| {
                    engine_image.handle = new_images[i];
                    engine_image.view = create_image_view(
                        &self.context.device.handle,
                        new_images[i],
                        self.format,
                        vk::ImageAspectFlags::COLOR,
                    )
                    .expect("Failed to create image view for swapchain image");
                });
        } else {
            println!("Creating sync objects");

            let command_buffers = unsafe {
                self.context.device.handle.allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_pool(self.command_pool)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        .command_buffer_count(new_images_len as u32),
                )
            }?;
            // fill swapchain.images with command buffers and sync objects.
            self.images = new_images
                .into_iter()
                .enumerate()
                .map(|(i, new_image)| EngineSwapchainImage {
                    handle: new_image,
                    view: create_image_view(
                        &self.context.device.handle,
                        new_image,
                        self.format,
                        vk::ImageAspectFlags::COLOR,
                    )
                    .expect("Failed to create image view for swapchain image"),
                    command_buffer: command_buffers[i],
                    image_available_semaphore: unsafe {
                        self.context
                            .device
                            .handle
                            .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                            .unwrap()
                    },
                    render_finished_semaphore: unsafe {
                        self.context
                            .device
                            .handle
                            .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                            .unwrap()
                    },
                    in_flight_fence: unsafe {
                        self.context
                            .device
                            .handle
                            .create_fence(
                                &vk::FenceCreateInfo::default()
                                    .flags(vk::FenceCreateFlags::SIGNALED),
                                None,
                            )
                            .unwrap()
                    },
                })
                .collect();
        }

        Ok(())
    }

    pub fn acquire_next_image(&mut self, image_available_semaphore: vk::Semaphore) -> Result<u32> {
        let (image_index, is_suboptimal) = unsafe {
            self.context.swapchain_loader.acquire_next_image2(
                &vk::AcquireNextImageInfoKHR::default()
                    .swapchain(self.handle)
                    .timeout(u64::MAX)
                    .semaphore(image_available_semaphore)
                    .device_mask(1),
            )?
        };
        if is_suboptimal {
            self.is_dirty = true;
        }
        Ok(image_index)
    }

    pub fn present(
        &mut self,
        image_index: u32,
        render_finished_semaphore: vk::Semaphore,
    ) -> Result<()> {
        let is_suboptimal = unsafe {
            match self.context.swapchain_loader.queue_present(
                self.context.device.queues[self.context.device.qfs.present.index as usize],
                &vk::PresentInfoKHR::default()
                    .wait_semaphores(&[render_finished_semaphore])
                    .swapchains(&[self.handle])
                    .image_indices(&[image_index]),
            ) {
                Ok(is_suboptimal) => is_suboptimal,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => true,
                Err(e) => return Err(e.into()),
            }
        };
        if is_suboptimal {
            self.is_dirty = true;
        }

        Ok(())
    }
}

impl Drop for EngineSwapchain {
    fn drop(&mut self) {
        unsafe {
            self.images.drain(..).for_each(|image| {
                self.context
                    .device
                    .handle
                    .destroy_image_view(image.view, None);
                self.context
                    .device
                    .handle
                    .destroy_semaphore(image.image_available_semaphore, None);
                self.context
                    .device
                    .handle
                    .destroy_semaphore(image.render_finished_semaphore, None);
                self.context
                    .device
                    .handle
                    .destroy_fence(image.in_flight_fence, None);
                self.context
                    .device
                    .handle
                    .free_command_buffers(self.command_pool, &[image.command_buffer]);
            });
            self.context
                .device
                .handle
                .destroy_command_pool(self.command_pool, None);
            self.context
                .swapchain_loader
                .destroy_swapchain(self.handle, None);
            self.context
                .surface_loader
                .destroy_surface(self.surface.handle, None);
        }
    }
}
