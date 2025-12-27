use wgpu::util::DeviceExt;

fn main() {
    println!("Testing WGPU types...");
    
    // This should work if the examples work
    let _image_copy_texture = wgpu::ImageCopyTexture {
        texture: &unimplemented!(),
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
    };
    
    let _image_copy_buffer = wgpu::ImageCopyBuffer {
        buffer: &unimplemented!(),
        layout: wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: None,
            rows_per_image: None,
        },
    };
    
    println!("✅ All WGPU types are available!");
}