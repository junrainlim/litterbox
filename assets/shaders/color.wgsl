
@group(0) @binding(0) 
var<uniform> size : vec2<u32>; // width, height
@group(0) @binding(1) 
var<storage, read_write> input: array<Cell>;
@group(0) @binding(2)
var<storage, read_write> output: array<Cell>;
@group(0) @binding(3)
var texture: texture_storage_2d<rgba8unorm, read_write>;

struct Cell {
    alive: u32,
    color: vec4<f32>,
}

fn idx(location: vec2<i32>) -> i32 {
    return location.y * i32(size.x) + location.x;
}

fn get_cell(location: vec2<i32>) -> Cell {
    return input[idx(location)];
}
fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    return state;
}

@compute @workgroup_size(8, 8, 1)
fn color_cells(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(invocation_id.xy);
    let cell = get_cell(location);
    var color: vec4<f32> = cell.color;

    textureStore(texture, location, vec4<f32>(color.xyz, f32(cell.alive)));
}