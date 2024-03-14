struct VertexInput {
	@location(0) position: vec2<f32>,
	@location(1) uv: vec2<f32>,
}

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
	var ret: VertexOutput;

	ret.position = vec4<f32>(input.position, 0.0, 1.0);
	ret.uv = input.uv;

	return ret;
}

@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
	return textureSample(tex, samp, input.uv);
}
