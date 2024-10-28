#version 320 es

precision mediump float;

in vec2 uv;

uniform sampler2D tex;

out vec4 final_color;

void main() {
	final_color = texture(tex, uv);
}
