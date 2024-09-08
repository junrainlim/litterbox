#import "shaders/core.wgsl"::{Cell, randomFloat}

@group(0) @binding(0) 
var<uniform> size : vec2<u32>; // width, height
@group(0) @binding(1) 
var<storage, read_write> input: array<Cell>;
@group(0) @binding(2)
var<storage, read_write> output: array<Cell>;

fn idx(location: vec2<i32>) -> i32 {
    return location.y * i32(size.x) + location.x;
}

fn get_cell(location: vec2<i32>, offset_x: i32, offset_y: i32) -> Cell {
    let loc = ((location + vec2<i32>(offset_x, offset_y)) + vec2<i32>(size)) % vec2<i32>(size);
    return input[idx(location + vec2<i32>(offset_x, offset_y))];
}

@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) global_invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let location = vec2<i32>(global_invocation_id.xy);

    let randomNumber = randomFloat(global_invocation_id.y * num_workgroups.x + global_invocation_id.x + workgroup_id.x + workgroup_id.y + workgroup_id.z);
    var type_id = 0;
    var color = vec4(0., 0., 0., 1.);

    // Not sure where this number comes from (divide by 1.2?) comes from but it works
    if (global_invocation_id.y == (size.y / 2) - 1) || (global_invocation_id.x == 0 || global_invocation_id.x == size.x - 1) {
        type_id = 1;
        color.x = 1.;
    }
    else if randomNumber > 0.9 {
        type_id = 2;
        color.z = 1.;
    }

    input[idx(location)] = Cell(i32(type_id), color);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let location = vec2<i32>(global_invocation_id.xy);
    // let num_neighbors = count_neighbors_simple(location);
    let cell = get_cell(location, 0, 0);

    var result: Cell = cell;
    switch cell.type_id {
        // Treat default as id=0 (Air)
        case 0, default {
            let above = get_cell(location, 0, -1);
            if above.type_id == 2 {
                result = Cell(2, above.color);
            } else if above.type_id == 0 {
                let above_right = get_cell(location, 1, -1);
                if above_right.type_id == 2 {
                    result = Cell(2, above_right.color);
                } else {
                    let above_left = get_cell(location, -1, -1);
                    if above_left.type_id == 2 {
                        result = Cell(2, above_left.color);
                    }
                }
            }
        }
        // Wall
        case 1 {
            result = cell;
        }
        // Sand
        case 2 {
            let below = get_cell(location, 0, 1);
            if below.type_id == 0 {
                result = Cell(0, vec4(0.,0.,0.,1.));
            } else {
                let below_left = get_cell(location, -1, 1);
                if below_left.type_id == 0 {
                    result = Cell(0, vec4(0.,0.,0.,1.));
                } else {
                    let below_right = get_cell(location, 1, 1);
                    if below_right.type_id == 0 {
                        result = Cell(0, vec4(0,0.,0.,1.));
                    }
                }
            }
        }
    }

    output[idx(location)] = result;
}