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

struct ScreenSize {
    width: f32,
    height: f32
}

@group(0) @binding(0)
var<uniform> screen_size: ScreenSize;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(
        // in.color,
        in.pos.x / screen_size.width,
        in.pos.y / screen_size.height,
        0.0,
        1.0
    );
}
