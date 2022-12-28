use std::{array, collections::HashMap, mem};

use ash::vk;
use hell_core::config;
use hell_error::HellResult;

use crate::{vulkan::{VulkanContextRef, primitives::{VulkanDescriptorSetGroup, VulkanSwapchain,  VulkanRenderPass, VulkanImage, VulkanBuffer, VulkanSampler, DeviceMemoryMapGuard}, pipeline::{VulkanShader, VulkanPipeline}}, render_types::PerFrame};


fn get_aligned(operand: usize, alignment: usize) -> usize {
    (operand + (alignment - 1)) & !(alignment - 1)
}

fn get_aligned_range(offset: usize, size: usize, alignment: usize) -> (usize, usize) {
    (get_aligned(offset, size), get_aligned(offset, alignment))
}


#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GenericShaderScope {
    Global = 0,
    Instance = 1,
    Local = 2,
}

impl GenericShaderScope {
    pub const SCOPE_COUNT: usize = 3;

    pub const INIT_IDX_GLOBAL:   usize = 0;
    pub const INIT_IDX_INSTANCE: usize = 1;
    pub const INIT_IDX_LOCAL:    usize = 2;

    pub const SET_IDX_GLOBAL:   usize = 0;
    pub const SET_IDX_INSTANCE: usize = 0;

    pub fn set_idx(&self) -> Option<usize> {
        match self {
            GenericShaderScope::Global   => Some(Self::SET_IDX_GLOBAL),
            GenericShaderScope::Instance => Some(Self::SET_IDX_INSTANCE),
            _ => None,
        }
    }
}

// ----------------------------------------------


struct UniformLookup {
    pub entry_idx: usize,
}

// ----------------------------------------------

#[derive(Debug)]
struct UniformEntry {
    pub name: String,
    pub entry_idx: usize,
    pub set_idx: Option<usize>,
    pub offset: usize,
    pub size: usize,
}

// ----------------------------------------------------------------------------


#[allow(dead_code)]
pub struct GenericVulkanShaderBuilder {
    ctx: VulkanContextRef,

    ubo_alignment: usize,
    push_constant_stride: usize,
    shader_path: String,
    depth_test_enabled: bool,
    is_wireframe: bool,

    desc_pool_sizes: [vk::DescriptorPoolSize; Self::DESC_COUNT],
    desc_bindings: [Option<[vk::DescriptorSetLayoutBinding; Self::DESC_COUNT]>; Self::MAX_SET_COUNT],

    global_textures: Vec<VulkanImage>,

    uniform_lookup: HashMap<String, UniformLookup>,

    uniform_entries: [Vec<UniformEntry>; GenericShaderScope::SCOPE_COUNT],
    ubo_sizes: [usize; GenericShaderScope::SCOPE_COUNT],
}

impl GenericVulkanShaderBuilder {
    const MAX_SET_COUNT: usize = 2;
    const DESC_COUNT: usize = 2;

    const BINDING_IDX_UBO:     u32 = 0;
    const BINDING_IDX_SAMPLER: u32 = 1;

    const MAX_GLOBAL_TEX_COUNT: usize = 16;
}

impl GenericVulkanShaderBuilder {
    pub fn new(ctx: &VulkanContextRef, shader_path: impl Into<String>) -> Self {
        let pool_sizes = [
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(config::VULKAN_UBO_DESCRIPTOR_COUNT as u32)
                .build(),
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(config::VULKAN_SAMPLER_DESCRIPTOR_COUNT as u32)
                .build(),
        ];



        Self {
            ctx: ctx.clone(),

            ubo_alignment: config::VULKAN_NVIDIA_REQUIRED_ALIGNMENT,
            push_constant_stride: config::VULKAN_GUARANTEED_PUSH_CONSTANT_STRIDE,
            shader_path: shader_path.into(),
            depth_test_enabled: false,
            is_wireframe: false,

            desc_pool_sizes: pool_sizes,
            desc_bindings: Default::default(),

            global_textures: Vec::with_capacity(Self::MAX_GLOBAL_TEX_COUNT),

            uniform_lookup: HashMap::new(),
            uniform_entries: Default::default(),
            ubo_sizes: Default::default(),
        }
    }

    pub fn with_depth_test(mut self) -> Self {
        self.depth_test_enabled = true;
        self
    }

    pub fn with_wireframe(mut self) -> Self {
        self.is_wireframe = true;
        self
    }

    pub fn with_global_bindings(mut self) -> Self {
        let bindings = [
            // ubo
            vk::DescriptorSetLayoutBinding::builder()
                .binding(Self::BINDING_IDX_UBO)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1) // number of elements in array
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                .build(),
            // image sampler (if used)
            vk::DescriptorSetLayoutBinding::builder()
                .binding(Self::BINDING_IDX_SAMPLER)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build()
        ];

        self.desc_bindings[GenericShaderScope::SET_IDX_GLOBAL] = Some(bindings);
        self
    }

    pub fn with_instance_bindings(mut self) -> Self {
        let bindings = [
            // ubo
            vk::DescriptorSetLayoutBinding::builder()
                .binding(Self::BINDING_IDX_UBO)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1) // number of elements in array
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                .build(),
            // image sampler (if used)
            vk::DescriptorSetLayoutBinding::builder()
                .binding(Self::BINDING_IDX_SAMPLER)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build()
        ];

        self.desc_bindings[GenericShaderScope::SET_IDX_INSTANCE] = Some(bindings);
        self
    }

    fn push_uniform_create_info(&mut self, name: impl Into<String>, scope: GenericShaderScope, mut size: usize, is_sampler: bool) {
        let mut offset = if is_sampler { 0 } else { self.ubo_sizes[scope as usize] };
        let set_idx = scope.set_idx();
        let name = name.into();

        if scope != GenericShaderScope::Local {
            if is_sampler { size = 0 }
        } else {
            let range = get_aligned_range(offset, size, 4);
            offset = range.0;
            size = range.1;
        }


        let entry_list = self.uniform_entries[scope as usize];
        let entry_idx = entry_list.len();

        let entry = UniformEntry {
            name: name.clone(),
            entry_idx,
            set_idx,
            offset,
            size
        };

        entry_list.push(entry);
        println!("PUSH-UNIFORM: {:?}", entry);

        self.ubo_sizes[scope as usize] += size;
        self.uniform_lookup.insert(name, UniformLookup { entry_idx });
    }

    pub fn with_uniform<T>(mut self, name: impl Into<String>, scope: GenericShaderScope) -> Self {
        self.push_uniform_create_info(name, scope, mem::size_of::<T>(), false);
        self
    }
}

impl GenericVulkanShaderBuilder {
    pub fn build(self, swapchain: &VulkanSwapchain, render_pass: &VulkanRenderPass) -> HellResult<GenericVulkanShader> {
        let ctx = &self.ctx;
        let device = &self.ctx.device.handle;

        // create descriptor-pool
        // ----------------------
        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&self.desc_pool_sizes)
            // maximum number of descriptor sets that may be allocated
            .max_sets(config::MAX_DESCRIPTOR_SET_COUNT)
            .build();

        let desc_pool = unsafe{ device.create_descriptor_pool(&pool_info, None)? };

        let desc_sets: [_; Self::MAX_SET_COUNT] = array::from_fn(|idx| {
            if let Some (bindings) = self.desc_bindings[idx] {
                let layout = VulkanDescriptorSetGroup::create_descriptor_set_layout(device, &bindings).unwrap();
                VulkanDescriptorSetGroup::new(&self.ctx, layout, 1)
            } else {
                VulkanDescriptorSetGroup::new(ctx, vk::DescriptorSetLayout::default(), 0)
            }
        });

        let desc_layouts: [_; Self::MAX_SET_COUNT] = array::from_fn(|idx| {
            desc_sets[idx].layout
        });

        // pipeline
        // --------
        let shader = VulkanShader::from_file(ctx, &self.shader_path)?;
        let pipeline = VulkanPipeline::new(ctx, swapchain, shader, render_pass, &desc_layouts, self.depth_test_enabled, self.is_wireframe)?;

        let global_ubo_stride = self.calculate_ubo_stride(GenericShaderScope::Global);
        let instance_ubo_stride = self.calculate_ubo_stride(GenericShaderScope::Instance);
        // max count should be configurable
        let total_buffer_size = global_ubo_stride + (instance_ubo_stride * config::VULKAN_MAX_MATERIAL_COUNT);

        let buffer = VulkanBuffer::new(
            ctx,
            total_buffer_size as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::SharingMode::EXCLUSIVE,
            None,
        );

        let buffer_mem = buffer.map_memory(0, total_buffer_size, vk::MemoryMapFlags::empty())?;

        Ok(GenericVulkanShader {
            ctx: self.ctx,
            desc_pool,
            pipeline,

            buffer,
            buffer_mem,
        })
    }

    fn calculate_ubo_stride(&self, scope: GenericShaderScope) -> usize {
        let mut stride = 0;
        let ubo_size = self.ubo_sizes[scope as usize];

        while stride < ubo_size {
            stride += self.ubo_alignment;
        }

        stride
    }

    fn create_uniform(ctx: &VulkanContextRef, pool: vk::DescriptorPool, layout: vk::DescriptorSetLayout, buffers: &[VulkanBuffer], samplers: &[VulkanSampler]) -> HellResult<()> {
        // Allocate one set per frame
        let sets = VulkanDescriptorSetGroup::allocate_sets_for_layout(ctx, layout, pool)?;

        for (idx, s) in sets.iter().enumerate() {
            let buffer_info = [
                vk::DescriptorBufferInfo::builder()
                    .buffer(buffers[idx].buffer)
                    .offset(0)
                    .range(SpriteShaderGlobalUniformObject::device_size())
                    .build()
            ];

            let image_info = [
                vk::DescriptorImageInfo::builder()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(samplers[idx].view)
                    .sampler(samplers[idx].sampler)
                    .build()
            ];

            let write_descriptors = [
                vk::WriteDescriptorSet::builder()
                    .dst_set(*s)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&buffer_info)
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(*s)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&image_info)
                    .build()
            ];

            unsafe { ctx.device.handle.update_descriptor_sets(&write_descriptors, &[]); }
        }

        let sets: PerFrame<vk::DescriptorSet> = array::from_fn(|idx| sets[idx]);

        Ok(())
    }
}

// ----------------------------------------------------------------------------

#[allow(dead_code)]
pub struct GenericVulkanShader {
    ctx: VulkanContextRef,
    desc_pool: vk::DescriptorPool,

    pipeline: VulkanPipeline,

    buffer: VulkanBuffer,
    buffer_mem: DeviceMemoryMapGuard<'static>,
}
