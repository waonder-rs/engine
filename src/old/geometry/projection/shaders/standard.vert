#version 450
layout(push_constant) uniform Projection {
	mat4 projection;
} pc;

layout(location = 0) in vec3 position;

void main() {
	gl_Position = pc.projection * vec4(position, 1.0);
}
