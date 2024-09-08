
#import "shaders/core.wgsl"::{Cell}

@group(0) @binding(0) 
var<uniform> size : vec2<u32>; // width, height
@group(0) @binding(1) 
var<storage, read_write> input: array<Cell>;
@group(0) @binding(2)
var<storage, read_write> output: array<Cell>;
@group(0) @binding(3)
var texture: texture_storage_2d<rgba8unorm, read_write>;

fn idx(location: vec2<i32>) -> i32 {
    return location.y * i32(size.x) + location.x;
}

fn get_cell(location: vec2<i32>) -> Cell {
    return input[idx(location)];
}

@compute @workgroup_size(8, 8, 1)
fn color_cells(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    var location = vec2<i32>(global_invocation_id.xy);
    let cell = get_cell(location);
    var color = cell.color;
    color.y = f32(global_invocation_id.y / size.y / 4);

    textureStore(texture, location, color);
}