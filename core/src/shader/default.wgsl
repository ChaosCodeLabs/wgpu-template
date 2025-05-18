struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>
}

// ===== Vertex shader =====

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var output = VertexOutput(
        vec4<f32>(in.pos, 1.0),
        in.color
    );
    return output;
}

// ===== Fragment shader =====

struct ProgramUniform {
    screen_width: f32,
    screen_height: f32
}
struct PerFrameUniform {
    time: f32,
    delta: f32,
    mouse: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> program_uniform: ProgramUniform;

@group(0) @binding(1)
var<uniform> per_frame_uniform: PerFrameUniform;

const circle_radius: f32 = 100.0;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let bg = vec4<f32>(
        // in.color,
        in.pos.x / program_uniform.screen_width,
        in.pos.y / program_uniform.screen_height,
        in.color.b,
        1.0
    );
    let dist = distance(in.pos.xy, per_frame_uniform.mouse);
    let is_in_circle = step(dist, circle_radius);
    let circle_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    return mix(bg, circle_color, is_in_circle);
}
