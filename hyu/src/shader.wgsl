struct VertexInput {
	@location(0) position: vec2<f32>,
	@location(1) color: vec4<f32>,
}

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
	var ret: VertexOutput;

	ret.position = vec4<f32>(input.position, 0.0, 1.0);
	ret.color = input.color;

	return ret;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
	return input.color;
}
