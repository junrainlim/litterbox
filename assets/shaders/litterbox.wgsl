// The shader reads the previous frame's state from the `input` texture, and writes the new state of
// each pixel to the `output` texture. The textures are flipped each step to progress the
// simulation.
// Two textures are needed for the game of life as each pixel of step N depends on the state of its
// neighbors at step N-1.
@group(0) @binding(0) 
var input: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(1)
var output: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) 
var<storage, read_write> color: vec4<f32>;
@group(0) @binding(3) 
var<storage, read_write> alive: u32;

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

fn randomFloat(value: u32) -> f32 {
    return f32(hash(value)) / 4294967295.0;
}

@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let location = vec2<i32>(invocation_id.xy);

    let randomNumber = randomFloat(invocation_id.y * num_workgroups.x + invocation_id.x);
    let alive = randomNumber > 0.9;
    let color = vec4<f32>(f32(alive), 0., f32(alive), 1.0);
    textureStore(output, location, color);
}

fn is_alive(location: vec2<i32>, offset_x: i32, offset_y: i32) -> u32 {
    let value: vec4<f32> = textureLoad(input, location + vec2<i32>(offset_x, offset_y));
    return u32(value.x);
}

fn count_neighbors_simple(location: vec2<i32>) -> u32 {
    var result: u32 = 0u;
    for (var x: i32 = -1; x < 2; x++) {
        for (var y: i32 = -1; y < 2; y++) {
            if x == 0 && y == 0 {
                continue;
            }

            result += is_alive(location, x, y);
        }
    }
    return result;
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(invocation_id.xy);
    let is_alive = bool(is_alive(location, 0, 0));
    let num_neighbors = count_neighbors_simple(location);

    var result: u32 = 0u;

    if is_alive {
        result = ((u32((num_neighbors) == (2u))) | (u32((num_neighbors) == (3u))));
    } else {
        result = u32((num_neighbors) == (3u));
    }

    let color = vec4<f32>(f32(result), 0, f32(result), 1.0);
    textureStore(output, location, color);
}