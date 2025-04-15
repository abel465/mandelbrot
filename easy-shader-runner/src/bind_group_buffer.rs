pub struct BufferDescriptor<'a> {
    pub data: &'a [u8],
    pub read_only: bool,
    pub shader_stages: wgpu::ShaderStages,
}
