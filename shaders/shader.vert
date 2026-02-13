#version 450

// Input : vertex attributes (from vertex buffer)
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec2 inTexCoords;
layout(location = 2) in vec3 inNormal;
layout(location = 3) in vec3 inColor;

// Uniforms : matrices MVP
layout(binding = 0) uniform UniformBufferObject {
	mat4 model;
	mat4 view;
	mat4 proj;
} ubo;

// Output : to fragment shader
layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragTextCoords;
layout(location = 2) out vec3 fragNormal;

void main() {
	// MVP transform
	gl_Position = ubo.proj * ubo.view * ubo.model * vec4(inPosition, 1.0);

	// Pass-through to fragment shader
	fragColor = inColor;
	fragTextCoords = inTexCoords;

	// Normal transform
	fragNormal = mat3(ubo.model) * inNormal;
}