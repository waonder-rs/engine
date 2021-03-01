#version 450
layout(location = 0) out vec4 color;

void main() {
	float depth = pow((1.0 - gl_FragCoord.z) * 0.5, 7.0) * 1000.0;

	color = vec4(depth, depth, depth, 1.0);
}
