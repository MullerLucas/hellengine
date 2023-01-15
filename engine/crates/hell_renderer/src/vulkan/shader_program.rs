#![allow(dead_code)]
#![allow(unused)]

use std::{array, collections::HashMap, mem};

use ash::vk;
use hell_collections::DynArray;
use hell_core::config;
use hell_error::{HellResult, HellError, HellErrorKind, HellErrorHelper, OptToHellErr};

use crate::{vulkan::{VulkanContextRef, primitives::{VulkanDescriptorSetGroup, VulkanSwapchain,  VulkanRenderPass, VulkanImage, VulkanBuffer, VulkanMemoryMap, VulkanCommands, VulkanSampler, VulkanTexture, VulkanCommandBuffer}, pipeline::{VulkanShader, VulkanPipeline}, VulkanFrame}, resources::{ResourceHandle, TextureManager}, render_types::{PerFrame, ValueRange, MemRange, NumberFormat}};





// ----------------------------------------------------------------------------

pub const fn get_aligned(operand: usize, alignment: usize) -> usize{
    (operand + (alignment - 1)) & !(alignment - 1)
}

pub const fn get_aligned_range(offset: usize, size: usize, alignment: usize) -> ValueRange<usize> {
    ValueRange::new(get_aligned(offset, size), get_aligned(offset, alignment))
}

// ----------------------------------------------------------------------------

pub type PerScope<T> = [T; ShaderScope::SCOPE_COUNT];
pub type PerSet<T> = DynArray<T, {ShaderScope::SCOPE_COUNT}>;


// ----------------------------------------------------------------------------

#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShaderScope {
    Global = 0,
    Shared = 1,
    Instance = 2,
    Local = 3,
}

impl From<usize> for ShaderScope {
    fn from(val: usize) -> Self {
        match val {
            0 => ShaderScope::Global,
            1 => ShaderScope::Shared,
            2 => ShaderScope::Instance,
            3 => ShaderScope::Local,
            _ => panic!("invalid scope")
        }
    }
}

impl ShaderScope {
    pub const ALL_SCOPES: &[ShaderScope] = &[
        Self::Global,
        Self::Shared,
        Self::Instance,
        Self::Local,
    ];

    pub const SCOPE_COUNT: usize = 4;

    pub const INIT_IDX_GLOBAL:   usize = 0;
    pub const INIT_IDX_INSTANCE: usize = 1;
    pub const INIT_IDX_LOCAL:    usize = 2;
}


// ----------------------------------------------


#[derive(Debug, Clone, Copy)]
pub struct UniformHandle {
    pub scope: ShaderScope,
    pub idx: usize,
}

impl UniformHandle {
    pub fn new(scope: ShaderScope, idx: usize) -> Self {
        Self { scope, idx }
    }
}


// ----------------------------------------------


#[derive(Debug)]
pub struct UniformInfo {
    pub name: String,
    pub scope: ShaderScope,
    pub idx: usize,
    pub range: MemRange,
}

impl UniformInfo {
    pub fn new_uniform(name: impl Into<String>, scope: ShaderScope, idx: usize, range: MemRange) -> Self {
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
            scope: ShaderScope::Local,
            idx: entry_idx,
            range,
        }
    }
}


// ----------------------------------------------


#[derive(Default, Debug)]
pub struct ScopeState {
    pub idx: usize,
    pub buffer_offset: usize,
    pub buffer_stride: usize,
    pub buffer_desc_sets: PerFrame<vk::DescriptorSet>,
    pub textures: DynArray<ResourceHandle, {config::VULKAN_MAX_SAMPLERS_PER_SHADER}>,
}

impl ScopeState {
    pub fn buffer_desc_set(&self, frame_idx: usize) -> vk::DescriptorSet {
        self.buffer_desc_sets[frame_idx]
    }

    pub fn textures(&self) -> &[ResourceHandle] {
        self.textures.as_slice()
    }
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
pub struct ShaderProgramBuilder {
    ctx: VulkanContextRef,
    ubo_alignment: usize,
    push_constant_stride: usize,
    depth_test_enabled: bool,
    is_wireframe: bool,
    shader_path: String,
    attributes: DynArray<AttributeInfo, { Self::MAX_ATTRIBUTE_COUNT }>,
    use_set: PerScope<bool>,
    sampler_counts: PerScope<usize>,
    uniforms: PerScope<Vec<UniformInfo>>,
    uniform_lookups: HashMap<String, UniformHandle>,
    scope_sizes: PerScope<usize>,
    scope_entry_count: PerScope<usize>,
    global_tex: Vec<ResourceHandle>,
    push_constant_ranges: Vec<MemRange>,
}

impl ShaderProgramBuilder {
    const DESC_COUNT: usize = 2;
    const BINDING_IDX_UBO:     u32 = 0;
    const BINDING_IDX_SAMPLER: u32 = 1;
    const MAX_GLOBAL_TEX_COUNT: usize = 16;
    const MAX_ATTRIBUTE_COUNT: usize = 32;
}

impl ShaderProgramBuilder {
    pub fn new(ctx: &VulkanContextRef, shader_path: impl Into<String>) -> Self {
        let scope_limits: [_; ShaderScope::SCOPE_COUNT] = [
            1,
            config::VULKAN_MAX_MATERIAL_COUNT,
            config::VULKAN_MAX_MATERIAL_COUNT,
            config::VULKAN_MAX_MATERIAL_COUNT,
        ];

        Self {
            ctx: ctx.clone(),
            ubo_alignment: config::VULKAN_NVIDIA_REQUIRED_ALIGNMENT,
            push_constant_stride: config::VULKAN_GUARANTEED_PUSH_CONSTANT_STRIDE,
            depth_test_enabled: false,
            is_wireframe: false,
            shader_path: shader_path.into(),
            attributes: DynArray::from_default(),
            uniform_lookups: HashMap::new(),
            use_set: Default::default(),
            uniforms: Default::default(),
            sampler_counts: Default::default(),
            scope_sizes: Default::default(),
            scope_entry_count: scope_limits,
            global_tex: Vec::with_capacity(Self::MAX_GLOBAL_TEX_COUNT),
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
        self.attributes.push(AttributeInfo {
            format,
            binding: 0,
            location: self.attributes.len()
        });

        self
    }

    // ------------------------------------------------------------------------

    fn push_uniform(&mut self, name: impl Into<String>, scope: ShaderScope, mut size: usize, is_sampler: bool) -> HellResult<()> {
        self.use_set[scope as usize] = true;

        let uniforms = &mut self.uniforms[scope as usize];
        let idx = uniforms.len();
        let mut offset = self.scope_sizes[scope as usize];
        let name = name.into();

        if is_sampler {
            offset = 0;
            size = 0;
        }

        let range = MemRange::new(offset, size);
        let info = UniformInfo::new_uniform(&name, scope, idx, range);

        // TODO: push constants
        // let range = get_aligned_range(offset, size, 4);
        // let info = UniformInfo::new_push_constant(&name, idx, range);
        // self.push_constant_ranges.push(info.range);
        // info

        println!("PUSH-UNIFORM: {:?}", info);
        // NOTE: use final size stored in info struct
        self.scope_sizes[scope as usize] += info.range.range;
        uniforms.push(info);
        self.uniform_lookups.insert(name, UniformHandle::new(scope, idx));

        Ok(())
    }

    // ------------------------------------------

    /// Adds a named uniform to the provided *Scope*
    pub fn with_uniform<T>(mut self, name: impl Into<String>, scope: ShaderScope) -> Self {
        self.push_uniform(name, scope, mem::size_of::<T>(), false);
        self
    }

    pub fn with_global_uniform<T>(self, name: impl Into<String>) -> Self {
        self.with_uniform::<T>(name, ShaderScope::Global)
    }

    pub fn with_shared_uniform<T>(self, name: impl Into<String>) -> Self {
        self.with_uniform::<T>(name, ShaderScope::Shared)
    }

    pub fn with_instance_uniform<T>(self, name: impl Into<String>) -> Self {
        self.with_uniform::<T>(name, ShaderScope::Instance)
    }

    pub fn with_local_uniform<T>(self, name: impl Into<String>) -> Self {
        self.with_uniform::<T>(name, ShaderScope::Local)
    }

    // ------------------------------------------

    /// Adds a named sampler to the provided *Scope*.
    pub fn with_sampler(mut self, name: impl Into<String>, scope: ShaderScope) -> HellResult<Self> {
        self.sampler_counts[scope as usize] += 1;
        self.push_uniform(name, scope, 0, true);
        Ok(self)
    }

    pub fn with_global_sampler(mut self, name: impl Into<String>, texture: ResourceHandle) -> HellResult<Self> {
        self.global_tex.push(texture);
        self.with_sampler(name, ShaderScope::Global)
    }

    pub fn with_pass_sampler(self, name: impl Into<String>) -> HellResult<Self> {
        self.with_sampler(name, ShaderScope::Shared)
    }

    pub fn with_instance_sampler(self, name: impl Into<String>) -> HellResult<Self> {
        self.with_sampler(name, ShaderScope::Instance)
    }

    pub fn with_local_sampler(self, name: impl Into<String>) -> HellResult<Self> {
        self.with_sampler(name, ShaderScope::Local)
    }

    // ------------------------------------------------------------------------

    /// Converts the *ShaderProgramBuilder* an usable *ShaderProgram*.
    pub fn build(mut self, swapchain: &VulkanSwapchain, render_pass: &VulkanRenderPass) -> HellResult<ShaderProgram> {
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

        // determine used sets: descriptor-layouts + mem-ranges
        // ----------------------------------------------------
        let mut set_desc_layouts: DynArray<vk::DescriptorSetLayout, {ShaderScope::SCOPE_COUNT}> = DynArray::from_default();
        let mut scope_desc_layouts: PerScope<Option<vk::DescriptorSetLayout>> = Default::default();
        let mut scope_ranges: PerScope<_> = Default::default();
        let mut total_buffer_size = 0;
        let mut scope_strides: PerScope<usize> = Default::default();
        let mut set_idx = 0;
        let mut scope_set_mapping: PerScope<Option<_>> = Default::default();

        for (idx, use_set) in self.use_set.iter().enumerate() {
            // sets have to be contigous -> there can't be a set 3 when there is no set 2
            if !use_set { continue; }
            println!("creating layoutfor set '{}'", idx);

            // scope-set-mapping
            // -----------------
            scope_set_mapping[idx] = Some(set_idx);
            set_idx += 1;

            // calculate ranges
            // ----------------
            let entry_count = self.scope_entry_count[idx];
            let ubo_stride = self.calculate_ubo_stride(ShaderScope::from(idx));
            let scope_size = ubo_stride * entry_count;
            scope_strides[idx] =  ubo_stride;
            scope_ranges[idx] = Some(MemRange::new(total_buffer_size, scope_size));
            total_buffer_size += scope_size;

            // create layout
            // -------------
            let sampler_count = self.sampler_counts[idx];
            let mut bindings: DynArray<vk::DescriptorSetLayoutBinding, 2> = DynArray::from_default();
            // one ubo
            bindings.push(
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(Self::BINDING_IDX_UBO)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1) // number of elements in array
                    .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                    .build()
            );

            // multiple samplers
            if sampler_count > 0 {
                bindings.push(
                    // multiple textures
                    vk::DescriptorSetLayoutBinding::builder()
                        .binding(Self::BINDING_IDX_SAMPLER)
                        .descriptor_count(sampler_count as u32)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                        .build()
                );
            }

            let layout = VulkanDescriptorSetGroup::create_descriptor_set_layout(device, bindings.as_slice())?;
            set_desc_layouts.push(layout);
            scope_desc_layouts[idx] = Some(layout);
        }

        // allocate buffer
        // ---------------
        // HACK: handle this differentyl
        debug_assert!(total_buffer_size > 0);
        let mut buffer = VulkanBuffer::from_uniform(ctx, total_buffer_size)?;
        buffer.mem.map_memory(0, total_buffer_size, vk::MemoryMapFlags::empty())?;

        // create descriptor-sets
        // ----------------------
        let global_layout = set_desc_layouts[ShaderScope::Global as usize];
        let global_desc_sets = VulkanDescriptorSetGroup::allocate_sets_for_layout(ctx, global_layout, desc_pool)?;

        // create pipeline
        // ---------------
        let shader = VulkanShader::from_file(ctx, &self.shader_path)?;
        let pipeline = VulkanPipeline::new(ctx, swapchain, shader, render_pass, &vert_binding_desc, vert_attrb_desc.as_slice(), set_desc_layouts.as_slice(), self.depth_test_enabled, self.is_wireframe)?;

        // scope states
        // ------------
        // self.global_tex.shrink_to_fit();
        let global_state = ScopeState {
            idx: 0,
            buffer_offset: 0,
            buffer_stride: scope_strides[ShaderScope::Global as usize],
            buffer_desc_sets: global_desc_sets,
            textures: DynArray::from(self.global_tex.as_slice())
        };
        let scope_states: PerScope<Vec<ScopeState>> = [
            vec![global_state],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ];

        Ok(ShaderProgram {
            ctx: self.ctx,
            scope_desc_layouts,
            desc_pool,
            scope_ranges,
            uniforms: self.uniforms,
            uniform_lookups: self.uniform_lookups,
            pipeline,
            buffer,
            sampler_counts: self.sampler_counts,
            scope_sizes: self.scope_sizes,
            scope_strides,
            scope_set_mapping,
            bound_scope: ShaderScope::Global,
            bound_entry: 0,
            bound_offset: 0,
            scope_entry_states: scope_states,
            global_entry: ResourceHandle::new(0),
        })
    }

    fn calculate_ubo_stride(&self, scope: ShaderScope) -> usize {
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
#[derive(Debug)]
pub struct ShaderProgram {
    ctx: VulkanContextRef,
    pub pipeline: VulkanPipeline,
    desc_pool: vk::DescriptorPool,
    uniform_lookups: HashMap<String, UniformHandle>,
    uniforms: PerScope<Vec<UniformInfo>>,
    buffer: VulkanBuffer,
    scope_ranges: PerScope<Option<MemRange>>,
    scope_desc_layouts: PerScope<Option<vk::DescriptorSetLayout>>,
    sampler_counts: PerScope<usize>,
    scope_sizes: PerScope<usize>, // actual size
    scope_strides: PerScope<usize>, // paddes size
    scope_set_mapping: PerScope<Option<usize>>,
    bound_scope: ShaderScope,
    bound_entry: usize,
    bound_offset: usize,
    scope_entry_states: PerScope<Vec<ScopeState>>,
    global_entry: ResourceHandle,
}

impl Drop for ShaderProgram {
    /// Drops the *ShaderProgram*. Cleans up all used Vulkan resources.
    fn drop(&mut self) {
        unsafe {
            // TODO:
            self.ctx.device.handle.destroy_descriptor_pool(self.desc_pool, None);
            self.scope_desc_layouts.as_slice().iter().filter_map(|l| l.as_ref()).for_each(|l| {
                self.ctx.device.handle.destroy_descriptor_set_layout(*l, None);
            });
        }
    }
}

impl ShaderProgram {
    pub fn uniform_handle(&self, name: &str) -> Option<UniformHandle> {
        self.uniform_lookups.get(name).copied()
    }

    pub fn uniform_handle_res(&self, name: &str) -> HellResult<UniformHandle> {
        self.uniform_handle(name).ok_or_render_herr("failed to get uniform")
    }

    // ------------------------------------------------------------------------

    // Binds the provided scope to be used.
    // If *scope* == ShaderScope::Global => *entry_idx* must be 0
    pub fn bind_scope(&mut self, scope: ShaderScope, entry_idx: usize) {
        debug_assert!((scope != ShaderScope::Global) || (entry_idx == 0));

        let state = &self.scope_entry_states[scope as usize][entry_idx];
        self.bound_scope = scope;
        self.bound_entry = entry_idx;
        self.bound_offset = state.buffer_offset;
    }

    pub fn bind_global(&mut self) {
        self.bind_scope(ShaderScope::Global, 0)
    }

    pub fn bind_shared(&mut self, entry_idx: usize) {
        self.bind_scope(ShaderScope::Shared, entry_idx)
    }

    pub fn bind_instance(&mut self, entry_idx: usize) {
        self.bind_scope(ShaderScope::Instance, entry_idx)
    }

    pub fn bind_local(&mut self, entry_idx: usize) {
        self.bind_scope(ShaderScope::Local, entry_idx)
    }

    // ------------------------------------------------------------------------

    pub fn set_uniform<T>(&mut self, handle: UniformHandle, value: &[T]) -> HellResult<()> {
        let uniform = &self.uniforms[handle.scope as usize][handle.idx];
        let offset = self.bound_offset + uniform.range.offset;

        self.buffer.mem
            .mapped_memory_mut()?
            .copy_from_nonoverlapping(value, offset as isize);
        Ok(())
    }

    // ------------------------------------------------------------------------

    fn calc_buffer_offset_and_size(&self, scope: ShaderScope, instance_idx: usize) -> HellResult<(usize, usize)> {
        let set_range = self.scope_ranges[scope as usize].ok_or_render_herr("failed to get scope range")?;
        let stride = self.scope_strides[scope as usize];
        let offset = set_range.offset + (stride * instance_idx);

        Ok((offset, stride))
    }

    pub fn acquire_scope_resource(&mut self, scope: ShaderScope, tex: &[ResourceHandle]) -> HellResult<ResourceHandle> {
        debug_assert_ne!(scope, ShaderScope::Global);

        let layout = self.scope_desc_layouts[scope as usize].ok_or_render_herr("failed to get scope desc-layout")?;
        let sampler_count = self.sampler_counts[scope as usize];
        debug_assert_eq!(tex.len(), sampler_count);

        let states = &self.scope_entry_states[scope as usize];
        let idx = states.len();
        let desc_sets = VulkanDescriptorSetGroup::allocate_sets_for_layout(&self.ctx, layout, self.desc_pool)?;
        let (offset, stride) = self.calc_buffer_offset_and_size(scope, idx)?;
        let mut textures = DynArray::from_default();
        tex.iter().for_each(|t| textures.push(*t));

        let state = ScopeState {
            idx,
            buffer_offset: offset,
            buffer_stride: stride,
            buffer_desc_sets: desc_sets,
            textures,
        };

        self.scope_entry_states[scope as usize].push(state);
        Ok(ResourceHandle::new(idx))
    }

    pub fn acquire_shared_resource(&mut self, tex: &[ResourceHandle]) -> HellResult<ResourceHandle> {
        self.acquire_scope_resource(ShaderScope::Shared, tex)
    }

    pub fn acquire_instance_resource(&mut self, tex: &[ResourceHandle]) -> HellResult<ResourceHandle> {
        self.acquire_scope_resource(ShaderScope::Instance, tex)
    }

    pub fn acquire_local_resource(&mut self, tex: &[ResourceHandle]) -> HellResult<ResourceHandle> {
        self.acquire_scope_resource(ShaderScope::Local, tex)
    }

    // ------------------------------------------------------------------------

    pub fn apply_scope_intern(&self, scope: ShaderScope, frame: &VulkanFrame, tex_man: &TextureManager, entry: ResourceHandle) -> HellResult<()> {
        let state = self.scope_entry_states[scope as usize].get(entry.idx).ok_or_render_herr("failed to get scope state")?;
        let buff_offset = state.buffer_offset;
        let buff_stride = state.buffer_stride;
        let desc_set = state.buffer_desc_set(frame.idx());
        let tex_handles = state.textures();

        let mut write_desc: DynArray<vk::WriteDescriptorSet, 2> = DynArray::from_default();

        // add buffer writes
        // -----------------
        let buffer_infos = [
            vk::DescriptorBufferInfo::builder()
                .buffer(self.buffer.handle)
                .offset(state.buffer_offset as u64)
                .range(state.buffer_stride as u64)
                .build()
        ];
        write_desc.push(
            vk::WriteDescriptorSet::builder()
                .dst_set(desc_set)
                .dst_binding(0) // corresponds to shader binding
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos)
                .build()
        );

        // add image writes
        // -----------------
        let sampler_count = self.sampler_counts[scope as usize];
        if sampler_count > 0 {
            if tex_handles.len() != sampler_count {
                return Err(HellErrorHelper::render_msg_err("sampler-count and tex-count do not match"));
            }

            let mut image_infos: DynArray<vk::DescriptorImageInfo, {config::VULKAN_SHADER_MAX_GLOBAL_TEXTURES}> = DynArray::from_default();
            for (idx, handle) in tex_handles.iter().enumerate() {
                let tex = tex_man.texture_res(*handle)?;

                image_infos.push(
                    vk::DescriptorImageInfo::builder()
                        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image_view(tex.img.view)
                        .sampler(tex.sampler.handle)
                        .build()
                );
            }

            if !image_infos.is_empty() {
                write_desc.push(
                    vk::WriteDescriptorSet::builder()
                        .dst_set(desc_set)
                        .dst_binding(1)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(image_infos.as_slice())
                        .build()
                );
            }
        }

        // update descriptor sets
        // ----------------------
        unsafe { self.ctx.device.handle.update_descriptor_sets(write_desc.as_slice(), &[]); }

        let cmd_buff = frame.gfx_cmd_buffer();
        let first_set = self.scope_set_mapping[scope as usize].ok_or_render_herr("failed to get scope mapping")? as u32;
        cmd_buff.cmd_bind_descriptor_sets(&self.ctx, vk::PipelineBindPoint::GRAPHICS, self.pipeline.layout, first_set, &[desc_set], &[]);

        Ok(())
    }

    pub fn apply_global_scope(&self, frame: &VulkanFrame, tex_man: &TextureManager) -> HellResult<()> {
        self.apply_scope_intern(ShaderScope::Global, frame, tex_man, self.global_entry)
    }

    pub fn apply_shared_scope(&self, frame: &VulkanFrame, tex_man: &TextureManager, entry: ResourceHandle) -> HellResult<()> {
        self.apply_scope_intern(ShaderScope::Shared, frame, tex_man, entry)
    }

    pub fn apply_instance_scope(&self, frame: &VulkanFrame, tex_man: &TextureManager, entry: ResourceHandle) -> HellResult<()> {
        self.apply_scope_intern(ShaderScope::Instance, frame, tex_man, entry)
    }

    pub fn apply_local_scope(&self, frame: &VulkanFrame, tex_man: &TextureManager, entry: ResourceHandle) -> HellResult<()> {
        self.apply_scope_intern(ShaderScope::Local, frame, tex_man, entry)
    }
}