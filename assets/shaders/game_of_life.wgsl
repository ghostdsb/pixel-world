const AIR_COLOR = vec4<f32>(0.02, 0.02, 0.02, 1.0);
const SAND_COLOR = vec4<f32>(0.8, 0.8, 0.2, 1.0); 
const WATER_COLOR = vec4<f32>(0.2, 0.2, 0.8, 1.0);
const ROCK_COLOR = vec4<f32>(0.2, 0.6, 0.6, 1.0);

@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

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
    var color = AIR_COLOR;
    let randomNumber = randomFloat(invocation_id.y * num_workgroups.x + invocation_id.x);
    let is_sand = randomNumber < 0.02;
    if(is_sand){
        color = SAND_COLOR;
    }
    textureStore(texture, location, color);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(invocation_id.xy);
    let current_particle_color = textureLoad(texture, location);
    var color = AIR_COLOR;
    if(compare_vectors(current_particle_color, SAND_COLOR)){
        // color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        textureStore(texture, location, AIR_COLOR);
        textureStore(texture, location + vec2<i32>(0, 1), SAND_COLOR);
    }
}

fn compare_vectors(v1: vec4<f32>, v2: vec4<f32>) -> bool {
    return v1.x == v2.x && v1.y == v2.y && v1.z == v2.z;
}