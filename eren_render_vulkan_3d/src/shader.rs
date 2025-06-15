use ash::vk;
use bytemuck::cast_slice;

pub fn create_shader_module(
    device: &ash::Device,
    code: &[u8],
) -> Result<vk::ShaderModule, vk::Result> {
    assert_eq!(
        code.len() % 4,
        0,
        "SPIR-V bytecode must be aligned to 4 bytes"
    );

    let mut owned = Vec::with_capacity(code.len());
    owned.extend_from_slice(code);

    let code_u32 = cast_slice(&owned);

    let create_info = vk::ShaderModuleCreateInfo::default().code(code_u32);

    unsafe { device.create_shader_module(&create_info, None) }
}
