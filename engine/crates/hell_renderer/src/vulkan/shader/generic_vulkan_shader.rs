#![allow(dead_code)]
#![allow(unused)]

use std::{array, collections::HashMap, mem};

use ash::vk;
use hell_collections::DynArray;
use hell_core::config;
use hell_error::{HellResult, HellError, HellErrorKind, HellErrorHelper};

use crate::vulkan::{VulkanContextRef, primitives::{VulkanDescriptorSetGroup, VulkanSwapchain,  VulkanRenderPass, VulkanImage, VulkanBuffer, VulkanMemoryMap, VulkanCommands, VulkanSampler, VulkanTexture, VulkanCommandBuffer}, pipeline::{VulkanShader, VulkanPipeline}, VulkanFrame};

#[allow(non_camel_case_types)]
#[derive(Default, Debug, Clone, Copy)]
pub enum NumberFormat {
    #[default] UNDEFINED,
    R32G32_SFLOAT,
    R32G32B32_SFLOAT,
    R32G32B32A32_SFLOAT,
}

impl NumberFormat {
    const fn size_of<T>(count: usize) -> usize {
        std::mem::size_of::<T>() * count
    }

    pub const fn to_vk_format(&self) -> vk::Format {
        match self {
            NumberFormat::R32G32_SFLOAT       => vk::Format::R32G32_SFLOAT,
            NumberFormat::R32G32B32_SFLOAT    => vk::Format::R32G32B32_SFLOAT,
            NumberFormat::R32G32B32A32_SFLOAT => vk::Format::R32G32B32A32_SFLOAT,
            _ => vk::Format::UNDEFINED,
        }
    }

    pub const fn size(&self) -> usize {
        match self {
            NumberFormat::R32G32_SFLOAT       => Self::size_of::<f32>(2),
            NumberFormat::R32G32B32_SFLOAT    => Self::size_of::<f32>(3),
            NumberFormat::R32G32B32A32_SFLOAT => Self::size_of::<f32>(4),
            _ => 0,
        }
    }
}

// ----------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct ValueRange<T> {
    pub offset: T,
    pub range: T,
}

impl<T> ValueRange<T> {
    pub const fn new(offset: T, range: T) -> Self {
        Self { offset, range }
    }
}

pub type MemRange = ValueRange<usize>;

// ----------------------------------------------------------------------------

const fn get_aligned(operand: usize, alignment: usize) -> usize{
    (operand + (alignment - 1)) & !(alignment - 1)
}

const fn get_aligned_range(offset: usize, size: usize, alignment: usize) -> ValueRange<usize> {
    ValueRange::new(get_aligned(offset, size), get_aligned(offset, alignment))
}

// ----------------------------------------------------------------------------


type PerScope<T> = [T; GenericShaderScope::SCOPE_COUNT];
type PerSet<T> = [T; GenericShaderScope::SET_COUNT];

#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GenericShaderScope {
    Global = 0,
    Instance = 1,
    Local = 2,
}

impl GenericShaderScope {
    pub const ALL_SCOPES: &[GenericShaderScope] = &[
        Self::Global,
        Self::Instance,
        Self::Local,
    ];

    pub const ALL_SETS: &[GenericShaderScope] = &[
        Self::Global,
        Self::Instance,
    ];

    pub const SCOPE_COUNT: usize = 3;
    pub const SET_COUNT: usize = 2;

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

    pub fn supports_samplers(&self) -> bool {
        *self != GenericShaderScope::Local
    }
}

// ----------------------------------------------


#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct UniformHandle {
    pub scope: GenericShaderScope,
    pub idx: usize,
}

impl UniformHandle {
    pub fn new(scope: GenericShaderScope, idx: usize) -> Self {
        Self { scope, idx }
    }
}

// ----------------------------------------------

#[allow(dead_code)]
#[derive(Debug)]
struct UniformInfo {
    pub name: String,
    pub scope: GenericShaderScope,
    pub idx: usize,
    pub range: MemRange,
}

impl UniformInfo {
    pub fn new_uniform(name: impl Into<String>, scope: GenericShaderScope, idx: usize, range: MemRange) -> Self {
        Self {
            name: name.into(),
            scope,
            idx,
            range,
        }
    }

    pub fn new_push_constant(name: impl Into<String>, entry_idx: usize, range: MemRange) -> Self {
        Self {
            name: name.into(),
            scope: GenericShaderScope::Local,
            idx: entry_idx,
            range,
        }
    }
}

// ----------------------------------------------

#[derive(Default)]
pub struct InstanceState {
    pub is_valid: bool,
    pub idx: usize,
    pub offset: usize,
}

// ----------------------------------------------

#[derive(Default)]
pub struct AttributeInfo {
    pub format: NumberFormat,
    pub binding: usize,
    pub location: usize,
}

// ----------------------------------------------

#[allow(dead_code)]
pub struct GenericVulkanShaderBuilder {
    ctx: VulkanContextRef,

    attributes: DynArray<AttributeInfo, { Self::MAX_ATTRIBUTE_COUNT }>,

    ubo_alignment: usize,
    push_constant_stride: usize,
    shader_path: String,
    depth_test_enabled: bool,
    is_wireframe: bool,

    use_set: PerScope<bool>,
    // desc_bindings: [Option<[vk::DescriptorSetLayoutBinding; Self::DESC_COUNT]>; GenericShaderScope::SET_COUNT],

    global_tex: Vec<VulkanTexture>,
    tex_count: PerScope<usize>,

    uniform_lookups: HashMap<String, UniformHandle>,
    uniforms: PerScope<Vec<UniformInfo>>,
    scope_sizes: PerScope<usize>,
    push_constant_ranges: Vec<MemRange>,
}

impl GenericVulkanShaderBuilder {
    const DESC_COUNT: usize = 2;

    const BINDING_IDX_UBO:     u32 = 0;
    const BINDING_IDX_SAMPLER: u32 = 1;

    const MAX_GLOBAL_TEX_COUNT: usize = 16;

    const MAX_ATTRIBUTE_COUNT: usize = 32;
}

impl GenericVulkanShaderBuilder {
    pub fn new(ctx: &VulkanContextRef, shader_path: impl Into<String>) -> Self {
        Self {
            ctx: ctx.clone(),

            attributes: DynArray::from_default(),

            ubo_alignment: config::VULKAN_NVIDIA_REQUIRED_ALIGNMENT,
            push_constant_stride: config::VULKAN_GUARANTEED_PUSH_CONSTANT_STRIDE,
            shader_path: shader_path.into(),
            depth_test_enabled: false,
            is_wireframe: false,

            use_set: Default::default(),
            // desc_bindings: Default::default(),

            global_tex: Vec::with_capacity(Self::MAX_GLOBAL_TEX_COUNT),
            tex_count: Default::default(),

            uniform_lookups: HashMap::new(),
            uniforms: Default::default(),
            scope_sizes: Default::default(),
            push_constant_ranges: Vec::new(),
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

    pub fn with_attribute(mut self, format: NumberFormat) -> Self {
        println!("ADDING-ATTRIBUTE: '{:?}'", format);

        self.attributes.push(AttributeInfo {
            format,
            binding: 0,
            location: self.attributes.len()
        });

        self
    }


    // ------------------------------------------------------------------------


    pub fn with_set_bindings(mut self, scope: GenericShaderScope) -> HellResult<Self> {
        if let Some(set) = scope.set_idx() {
            self.use_set[set] = true;
            Ok(self)
        } else {
            Err(HellErrorHelper::render_msg_err("trying to use bindings for invalid set"))
        }
    }

    pub fn with_global_bindings(mut self) -> Self {
        self.with_set_bindings(GenericShaderScope::Global).unwrap()
    }

    pub fn with_instance_bindings(mut self) -> Self {
        self.with_set_bindings(GenericShaderScope::Instance).unwrap()
    }


    // ------------------------------------------------------------------------


    fn push_uniform(&mut self, name: impl Into<String>, scope: GenericShaderScope, mut size: usize, is_sampler: bool) -> HellResult<()> {
        let uniforms = &mut self.uniforms[scope as usize];
        let idx = uniforms.len();
        let mut offset = self.scope_sizes[scope as usize];
        let name = name.into();

        if is_sampler {
            offset = 0;
            size = 0;

            if !scope.supports_samplers() {
                return Err(HellErrorHelper::render_msg_err("trying to push sampler to unsuported scope"));
            }
        }

        let info = match scope {
            GenericShaderScope::Global |
            GenericShaderScope::Instance => {
                let range = MemRange::new(offset, size);
                UniformInfo::new_uniform(&name, scope, idx, range)
            }
            GenericShaderScope::Local => {
                let range = get_aligned_range(offset, size, 4);
                let info = UniformInfo::new_push_constant(&name, idx, range);
                self.push_constant_ranges.push(info.range);
                info
            }
        };

        println!("PUSH-UNIFORM: {:?}", info);
        // NOTE: use final size stored in info struct
        self.scope_sizes[scope as usize] += info.range.range;
        uniforms.push(info);
        self.uniform_lookups.insert(name, UniformHandle::new(scope, idx));

        Ok(())
    }

    pub fn with_uniform<T>(mut self, name: impl Into<String>, scope: GenericShaderScope) -> Self {
        self.push_uniform(name, scope, mem::size_of::<T>(), false);
        self
    }

    pub fn with_global_uniform<T>(self, name: impl Into<String>) -> Self {
        self.with_uniform::<T>(name, GenericShaderScope::Global)
    }

    pub fn with_sampler(mut self, name: impl Into<String>, scope: GenericShaderScope, cmds: &VulkanCommands) -> HellResult<Self> {
        self.tex_count[scope as usize] += 1;

        match scope {
            GenericShaderScope::Global => {
                self.global_tex.push(
                    VulkanTexture::new_default(&self.ctx, cmds)?
                );
            }
            GenericShaderScope::Instance => {

            },
            _ => { panic!("invalid sampler scope"); }
        }

        self.push_uniform(name, scope, 0, true);

        Ok(self)
    }

    pub fn with_global_sampler(self, name: impl Into<String>, cmds: &VulkanCommands) -> HellResult<Self> {
        self.with_sampler(name, GenericShaderScope::Global, cmds)
    }


    // ------------------------------------------------------------------------


    pub fn build(mut self, swapchain: &VulkanSwapchain, render_pass: &VulkanRenderPass) -> HellResult<GenericVulkanShader> {
        let ctx = &self.ctx;
        let device = &self.ctx.device.handle;

        // create vertex-data
        // ------------------
        let mut vert_stride = 0_usize;
        let mut vert_attrb_desc: DynArray<vk::VertexInputAttributeDescription, { Self::MAX_ATTRIBUTE_COUNT }> = DynArray::from_default();
        self.attributes.as_slice().iter().enumerate().for_each(|(idx, attr)| {
            vert_attrb_desc.push(vk::VertexInputAttributeDescription::builder()
                .location(idx as u32)
                .binding(0)
                .format(attr.format.to_vk_format())
                .offset(vert_stride as u32)
                .build()
            );
            vert_stride += attr.format.size();
        });

        let mut vert_binding_desc = [vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(vert_stride as u32)
            .build()
        ];

        // create descriptor-pool
        // ----------------------
        let desc_pool_sizes = [
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(config::VULKAN_UBO_DESCRIPTOR_COUNT as u32)
                .build(),
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(config::VULKAN_SAMPLER_DESCRIPTOR_COUNT as u32)
                .build(),
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&desc_pool_sizes)
            // maximum number of descriptor sets that may be allocated
            .max_sets(config::MAX_DESCRIPTOR_SET_COUNT as u32)
            .build();

        let desc_pool = unsafe{ device.create_descriptor_pool(&pool_info, None)? };
        let desc_sets: PerSet<VulkanDescriptorSetGroup> = array::from_fn(|idx| {
            if self.use_set[idx] {
                let tex_count = self.tex_count[idx];
                let bindings = [
                    // one ubo
                    vk::DescriptorSetLayoutBinding::builder()
                        .binding(Self::BINDING_IDX_UBO)
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .descriptor_count(1) // number of elements in array
                        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                        .build(),
                    // multiple textures
                    vk::DescriptorSetLayoutBinding::builder()
                        .binding(Self::BINDING_IDX_SAMPLER)
                        .descriptor_count(tex_count as u32)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                        .build(),
                ];

                let layout = VulkanDescriptorSetGroup::create_descriptor_set_layout(device, &bindings).unwrap();
                VulkanDescriptorSetGroup::new(&self.ctx, layout, 1)
            } else {
                VulkanDescriptorSetGroup::new(ctx, vk::DescriptorSetLayout::default(), 0)
            }
        });

        let desc_layouts: [_; GenericShaderScope::SET_COUNT] = array::from_fn(|idx| {
            desc_sets[idx].layout
        });

        // global ubos
        // -----------
        let global_ubo_stride = self.calculate_ubo_stride(GenericShaderScope::Global);
        let instance_ubo_stride = self.calculate_ubo_stride(GenericShaderScope::Instance);
        // max count should be configurable
        let total_buffer_size = global_ubo_stride + (instance_ubo_stride * config::VULKAN_MAX_MATERIAL_COUNT);

        let ubo_ranges = [
            // global
            MemRange::new(0, global_ubo_stride),
            // instance
            MemRange::new(global_ubo_stride, instance_ubo_stride),
        ];

        // allocate buffer
        // ---------------
        debug_assert!(total_buffer_size > 0);
        let mut buffer = VulkanBuffer::from_uniform(ctx, total_buffer_size)?;
        buffer.mem.map_memory(0, total_buffer_size, vk::MemoryMapFlags::empty())?;
        // let buffer_map = buffer.map_memory(0, total_buffer_size, vk::MemoryMapFlags::empty())?;

        // global textures
        // ---------------
        self.global_tex.shrink_to_fit();

        // create descriptor-sets
        // ----------------------
        let global_layout = desc_sets[GenericShaderScope::Global as usize].layout;
        let global_desc_sets = VulkanDescriptorSetGroup::allocate_sets_for_layout(ctx, global_layout, desc_pool)?;

        // create instance states
        // ----------------------
        let instance_states = array::from_fn(|_| InstanceState::default());

        // create pipeline
        // ---------------
        let shader = VulkanShader::from_file(ctx, &self.shader_path)?;
        // TODO: desc_layouts
        let pipeline = VulkanPipeline::new(ctx, swapchain, shader, render_pass, &vert_binding_desc, vert_attrb_desc.as_slice(), &desc_layouts[0..=0], self.depth_test_enabled, self.is_wireframe)?;




        Ok(GenericVulkanShader {
            ctx: self.ctx,
            desc_sets,
            desc_pool,
            global_desc_sets,
            ubo_ranges,
            uniforms: self.uniforms,
            uniform_lookups: self.uniform_lookups,
            global_tex: self.global_tex,
            pipeline,
            buffer,
            instance_states,

        })
    }

    fn calculate_ubo_stride(&self, scope: GenericShaderScope) -> usize {
        let mut stride = 0;
        let ubo_size = self.scope_sizes[scope as usize];

        while stride < ubo_size {
            stride += self.ubo_alignment;
        }

        stride
    }
}

// ----------------------------------------------------------------------------

#[allow(dead_code)]
pub struct GenericVulkanShader {
    ctx: VulkanContextRef,
    desc_sets: [VulkanDescriptorSetGroup; GenericShaderScope::SET_COUNT],
    desc_pool: vk::DescriptorPool,
    global_desc_sets: Vec<vk::DescriptorSet>,
    ubo_ranges: [MemRange; GenericShaderScope::SET_COUNT],
    uniform_lookups: HashMap<String, UniformHandle>,
    uniforms: PerScope<Vec<UniformInfo>>,
    global_tex: Vec<VulkanTexture>,
    pub pipeline: VulkanPipeline,
    buffer: VulkanBuffer,
    instance_states: [InstanceState; config::VULKAN_MAX_MATERIAL_COUNT],
}

impl Drop for GenericVulkanShader {
    fn drop(&mut self) {
        unsafe {
            // TODO:
            self.ctx.device.handle.destroy_descriptor_pool(self.desc_pool, None);
        }
    }
}

impl GenericVulkanShader {
    pub fn uniform_handle(&self, name: &str) -> Option<UniformHandle> {
        self.uniform_lookups.get(name).copied()
    }

    pub fn global_descriptor(&self, frame_idx: usize) -> vk::DescriptorSet {
        self.global_desc_sets[frame_idx]
    }


    // ------------------------------------------------------------------------


    pub fn set_uniform<T>(&mut self, handle: UniformHandle, value: &[T]) -> HellResult<()> {
        let uniform = &self.uniforms[handle.scope as usize][handle.idx];
        println!("SET-UNIFORM: '{:?}' - '{:?}'", handle, uniform);
        self.buffer.mem
            .mapped_memory_mut()?
            .copy_from_nonoverlapping(value, uniform.range.offset as isize);

        Ok(())
    }

    // ------------------------------------------------------------------------


    pub fn apply_scope(&self, scope: GenericShaderScope, frame: &VulkanFrame) -> HellResult<()> {
        println!("APPLY: {}", frame.idx());
        let set = self.global_desc_sets[frame.idx()];
        let ubo_range = &self.ubo_ranges[scope as usize];

        // update desc-set
        // ---------------
        let buffer_infos = [
            vk::DescriptorBufferInfo::builder()
                .buffer(self.buffer.handle)
                .offset(ubo_range.offset as u64)
                .range(ubo_range.range as u64)
                .build()
        ];

        let mut image_infos: DynArray<vk::DescriptorImageInfo, {config::VULKAN_SHADER_MAX_GLOBAL_TEXTURES}> = DynArray::from_default();
        for (idx, tex) in self.global_tex.iter().enumerate() {
            image_infos.push(
                vk::DescriptorImageInfo::builder()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(tex.img.view)
                    .sampler(tex.sampler.handle)
                    .build()
            );
        }

        let write_descriptors = [
            vk::WriteDescriptorSet::builder()
                .dst_set(set)
                .dst_binding(0) // corresponds to shader binding
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(set)
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(image_infos.as_slice())
                .build()
        ];

        unsafe { self.ctx.device.handle.update_descriptor_sets(&write_descriptors, &[]); }

        // bind desc-set
        // -------------
        let descriptor_set = [ self.global_descriptor(frame.idx()) ];
        let cmd_buff = frame.gfx_cmd_buffer();
        cmd_buff.cmd_bind_descriptor_sets(&self.ctx, vk::PipelineBindPoint::GRAPHICS, self.pipeline.layout, 0, &descriptor_set, &[]);

        Ok(())
    }

    pub fn apply_globals(&self, frame: &VulkanFrame) -> HellResult<()> {
        self.apply_scope(GenericShaderScope::Global, frame)
    }

    pub fn apply_locals(&self, frame: &VulkanFrame) -> HellResult<()> {
        self.apply_scope(GenericShaderScope::Local, frame)
    }
}
