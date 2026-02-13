#version 450

// Input: from vertex shader
layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoords;
layout(location = 2) in vec3 fragNormal;

// Uniforms: mix factor and texture
layout(binding = 1) uniform sampler2D texSampler;
layout(binding = 2) uniform MixFactorUBO {
	float mixValue;	// 0.0 = pure color, 1.0 = pure texture
} mixFactor;

// Output = final color
layout(location = 0) out vec4 outColor;

void main() {
	// get texture color
	vec4 texColor = texture(texSampler, fragTexCoords);

	// mix vertex & texture colors
	vec4 colorOnly = vec4(fragColor, 1.0);
	outColor = mix(colorOnly, texColor, mixFactor.mixValue);

	// simple lighting
	float lighting = max(dot(normalize(fragNormal), vec3(0.0, 1.0, 0.0)), 0.3);
	outColor *= lighting;
}