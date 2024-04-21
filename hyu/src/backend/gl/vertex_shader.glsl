#version 320 es

precision mediump float;

layout (location = 0) in vec2 in_pos;
layout (location = 1) in vec2 in_uv;

out vec2 uv;

void main() {
	gl_Position = vec4(in_pos.x, in_pos.y, 0.0, 1.0);
	uv = in_uv;
}
