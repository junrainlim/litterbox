@group(0) @binding(0) 
var<uniform> size : vec2<u32>; // width, height
@group(0) @binding(1) 
var<storage, read_write> input: array<Cell>;
@group(0) @binding(2)
var<storage, read_write> output: array<Cell>;

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

fn is_alive(location: vec2<i32>, offset_x: i32, offset_y: i32) -> u32 {
    var loc = ((location + vec2<i32>(offset_x, offset_y)) + vec2<i32>(size)) % vec2<i32>(size);
    return input[idx(loc)].alive;
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

fn randomFloat(value: u32) -> f32 {
    return f32(hash(value)) / 4294967295.0;
}


@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let location = vec2<i32>(invocation_id.xy);

    let randomNumber = randomFloat(invocation_id.y * num_workgroups.x + invocation_id.x + workgroup_id.x + workgroup_id.y + workgroup_id.z);
    let alive = randomNumber > 0.9;

    input[idx(location)] = Cell(u32(alive), vec4(0., 0., 1., f32(alive)));
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
    let num_neighbors = count_neighbors_simple(location);
    var cell = get_cell(location);
    let is_alive = bool(cell.alive);

    var result: u32 = 0u;

    if is_alive {
        result = ((u32((num_neighbors) == (2u))) | (u32((num_neighbors) == (3u))));
    } else {
        result = u32((num_neighbors) == (3u));
    }

    output[idx(location)] = Cell(result, cell.color);
}