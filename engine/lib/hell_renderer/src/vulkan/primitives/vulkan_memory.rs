use std::fmt::Debug;

use ash::vk;
use hell_core::error::{HellResult, HellError, HellErrorKind, HellErrorContent};
use crate::vulkan::VulkanContextRef;


pub type VulkanRawMemoryMap = VulkanMemoryMap<u8>;

// #[derive(Debug)]
pub struct VulkanDeviceMemory {
    ctx: VulkanContextRef,
    requirements: vk::MemoryRequirements,
    properties: vk::MemoryPropertyFlags,
    mem_map: Option<VulkanRawMemoryMap>,
    handle: vk::DeviceMemory,
}

impl Debug for VulkanDeviceMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VulkanDeviceMemory")
            .field("ctx", &self.ctx)
            .field("requirements", &self.requirements)
            .field("properties", &self.properties)
            .field("mem_map", &self.mem_map)
            .field("handle", &self.handle)
            .finish()
    }
}

// TODO: error handling
impl Drop for VulkanDeviceMemory {
    fn drop(&mut self) {
        unsafe {
            // unmap memory if mapped
            if self.mem_map.is_some() {
                self.unmap_memory().unwrap();
            }

            // free memory
            self.ctx.device.handle.free_memory(self.handle, None);
        }
    }
}

impl VulkanDeviceMemory {
    pub fn new(ctx: &VulkanContextRef, requirements: vk::MemoryRequirements, properties: vk::MemoryPropertyFlags) -> HellResult<Self> {
        let mem_type_idx = Self::find_memory_type(ctx, requirements.memory_type_bits, properties);

        let alloc_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            allocation_size: requirements.size,
            memory_type_index: mem_type_idx
        };

        let mem = unsafe { ctx.device.handle.allocate_memory(&alloc_info, None) }?;

        Ok(Self {
            ctx: ctx.clone(),
            handle: mem,
            requirements,
            properties,
            mem_map: None,
        })
    }

    pub fn find_memory_type(ctx: &VulkanContextRef, type_filter: u32, properties: vk::MemoryPropertyFlags) -> u32 {
        let mem_props = unsafe { ctx.instance.instance.get_physical_device_memory_properties(ctx.phys_device.phys_device) };

        for (i, mem_type) in mem_props.memory_types.iter().enumerate() {
            if (type_filter & (1 << i) > 0) && mem_type.property_flags.contains(properties)  {
                return i as u32;
            }
        }

        panic!("failed to find suitable memory-type");
    }

    pub fn requirements(&self) -> vk::MemoryRequirements {
        self.requirements
    }

    pub fn size(&self) -> usize {
        self.requirements.size as usize
    }

    pub fn alignment(&self) -> usize {
        self.requirements.alignment as usize
    }

    pub fn bind_to_buffer(&self, buffer: vk::Buffer) -> HellResult<()> {
        unsafe { self.ctx.device.handle.bind_buffer_memory(buffer, self.handle, 0) }?;
        Ok(())
    }

    pub fn bind_to_image(&self, img: vk::Image, offset: usize) -> HellResult<()> {
        unsafe {
            self.ctx.device.handle.bind_image_memory(img, self.handle, offset as vk::DeviceSize)?;
        }
        Ok(())
    }

    pub fn memory_is_mapped(&self) -> bool {
        self.mem_map.is_some()
    }

    pub fn map_memory(&mut self, offset: usize, size: usize, mem_map_flags: vk::MemoryMapFlags) -> HellResult<&mut VulkanRawMemoryMap> {
        if self.memory_is_mapped() {
            return Err(HellError::new(HellErrorKind::RenderError, HellErrorContent::Message("trying to map memory, but memory is already mapped ".to_string())));
        }

        self.mem_map = Some(
            VulkanRawMemoryMap::new(&self.ctx, self.handle, offset, size, mem_map_flags)?
        );
        Ok(self.mem_map.as_mut().unwrap())
    }

    pub fn unmap_memory(&mut self) -> HellResult<()> {
        if !self.memory_is_mapped() {
            return Err(HellError::new(HellErrorKind::RenderError, HellErrorContent::Message("trying to unmap memory, but memory is not mapped".to_string())));
        }

        unsafe { self.ctx.device.handle.unmap_memory(self.handle); }
        self.mem_map = None;
        Ok(())
    }

    pub fn mapped_memory(&self) -> HellResult<&VulkanRawMemoryMap> {
        self.mem_map.as_ref()
            .ok_or_else(|| HellError::new(HellErrorKind::RenderError, HellErrorContent::Message("failed to retrieve mapped memory".to_string())))
    }

    pub fn mapped_memory_mut(&mut self) -> HellResult<&mut VulkanRawMemoryMap> {
        self.mem_map.as_mut()
            .ok_or_else(|| HellError::new(HellErrorKind::RenderError, HellErrorContent::Message("failed to retrieved mapped memory mut".to_string())))
    }

    pub fn mapped_memory_opt(&self) -> Option<&VulkanRawMemoryMap> {
        self.mem_map.as_ref()
    }

    pub fn mapped_memory_mut_opt(&mut self) -> Option<&mut VulkanRawMemoryMap> {
        self.mem_map.as_mut()
    }
}

impl VulkanDeviceMemory {
    pub fn create_buffer_requirements(ctx: &VulkanContextRef, buffer: vk::Buffer) -> vk::MemoryRequirements {
        unsafe { ctx.device.handle.get_buffer_memory_requirements(buffer) }
    }
}



// ----------------------------------------------------------------------------
// VulkanMemoryMap
// ----------------------------------------------------------------------------

#[derive(Debug)]
#[allow(dead_code)]
pub struct VulkanMemoryMap<T> {
    data_ptr: *mut T,
    offset: usize,
    size: usize,
}

impl<T> VulkanMemoryMap<T> {
    pub fn new(ctx: &VulkanContextRef, mem: vk::DeviceMemory, offset: usize, size: usize, mem_map_flags: vk::MemoryMapFlags) -> HellResult<Self> {
        let data_ptr = unsafe { ctx.device.handle.map_memory(mem, offset as vk::DeviceSize, size as vk::DeviceSize, mem_map_flags)? } as *mut T;

        Ok(Self {
            data_ptr,
            offset,
            size,
        })
    }

    // NOTE: theoretically does not need to be &mut self - but its probably a good idea
    pub fn copy_from_nonoverlapping<V>(&mut self, src: &[V], offset: isize) {
        let align = std::mem::size_of::<V>();
        let count = src.len() * align;
        let src_ptr = src.as_ptr() as *const T;

        unsafe {
            self.data_ptr
                // NOTE: *offset* depends on the size of T
                .offset(offset)
                .copy_from_nonoverlapping(src_ptr, count);
        }
    }

    pub fn fill_with_value<V>(&mut self, val: V) {
        let align = std::mem::align_of::<T>();
        let size = (self.size / align) * self.size;
        let val = (&val) as *const V as *const T;

        (0..size)
            .into_iter()
            .step_by(align)
            .for_each(|off| { unsafe {
                self.data_ptr
                    // NOTE: *add* depends on the size of T
                    .add(off)
                    .copy_from_nonoverlapping(val, align);
            } })
    }
}
