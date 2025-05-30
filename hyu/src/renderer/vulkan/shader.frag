#version 450

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

layout(binding = 0) uniform sampler2D texture_sampler;

void main() {
	out_color = texture(texture_sampler, in_uv);
}
