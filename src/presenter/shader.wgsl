// 화면정렬 quad 에 CEF 텍스처를 textureSample 로 그린다 — cef-rs 공식 OSR 예제(shader.wgsl) 그대로.
struct VertexInput {
    @location(0) pos: vec4<f32>,
    @location(1) tex: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.pos = input.pos;
    output.tex = input.tex;
    return output;
}

@group(0) @binding(0) var tex0: texture_2d<f32>;
@group(0) @binding(1) var samp0: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex0, samp0, input.tex);
}
