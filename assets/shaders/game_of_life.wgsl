const AIR_COLOR = vec4<f32>(0.02, 0.02, 0.02, 1.0);
const SAND_COLOR = vec4<f32>(0.8, 0.8, 0.2, 1.0); 
const WATER_COLOR = vec4<f32>(0.2, 0.2, 0.8, 1.0);
const ROCK_COLOR = vec4<f32>(0.4, 0.4, 0.4, 1.0);

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
    let is_sand = randomNumber < 0.3;
    if(is_sand){
        // color = SAND_COLOR;
    }
    if(location.y > 650){
        color = ROCK_COLOR;
    }
    textureStore(texture, location, color);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    // let source = vec2<i32>(400, 0);
    // textureStore(texture, source, SAND_COLOR);

    let location = vec2<i32>(invocation_id.xy);
    let current_particle_color = textureLoad(texture, location);
    var color = AIR_COLOR;
    if(compare_vectors(current_particle_color, AIR_COLOR)){

    }else if(compare_vectors(current_particle_color, SAND_COLOR)){
        let color_below = textureLoad(texture, location + vec2<i32>(0, 1));
        let randomNumber = randomFloat(invocation_id.y + invocation_id.x);
        var x = 1;
        if(randomNumber < 0.5){
            x = -1;
        }
        let color_diagonally_below = textureLoad(texture, location + vec2<i32>(x, 1));

        if(compare_vectors(color_below, AIR_COLOR)){
            textureStore(texture, location, AIR_COLOR);
            textureStore(texture, location + vec2<i32>(0, 1), SAND_COLOR);
        }else if(compare_vectors(color_below, SAND_COLOR)){
            if(compare_vectors(color_diagonally_below, AIR_COLOR)){
                textureStore(texture, location, AIR_COLOR);
                textureStore(texture, location + vec2<i32>(x, 1), SAND_COLOR);
            }
        }else{
            
        }
    }
}

fn compare_vectors(v1: vec4<f32>, v2: vec4<f32>) -> bool {
    return distance(v1, v2) < 0.01;
}