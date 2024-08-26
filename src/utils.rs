use bevy::render::{render_resource::{Buffer, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice};

pub fn create_uniform_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(
    device: &RenderDevice,
    data: &[T],
    label: Option<&str>,
) -> Buffer {
    device.create_buffer_with_data(&BufferInitDescriptor {
        label,
        contents: bytemuck::cast_slice(data),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
}

pub fn create_storage_buffer_with_data<T: bytemuck::Pod + bytemuck::Zeroable>(
    device: &RenderDevice,
    data: &[T],
    label: Option<&str>,
) -> Buffer {
    device.create_buffer_with_data(&BufferInitDescriptor {
        label,
        contents: bytemuck::cast_slice(data),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    })
}
