#version 460

// Per-fragment - mesh dependent information.
layout(location = 0) in vec3 inFragPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec3 inUv;

// Per-frame information - CameraUniform struct
layout(set = 0, binding = 0) uniform CameraUniform {
    mat4 view;
    mat4 projection;
    vec3 position;
} camera;

layout(location = 0) out vec4 f_color;

void main() {
    // Default mesh color - no textures for now.
    vec3 objectColor = vec3(0.5, 0.5, 0.5);
    // Default light information - we aren't going to edit light information until we know more about it.
    vec3 lightPosition = vec3(-100.0, 200.0, 200.0);
    vec3 lightColor = vec3(1.0, 1.0, 1.0);

    // Ambient lighting - literally just a base color percentage.
    vec3 ambient = 0.1 * lightColor;

    // Diffuse lighting - brighter on faces that catch more light.
    // This is the dot product of the frag to the light, and the frag normal.
    vec3 norm = normalize(inNormal);
    vec3 lightDir = normalize(lightPosition - inFragPosition);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;

    // Specular lighting - shiny spots corresponding to reflected light toward the camera.
    // The direction from the fragment to the camera.
    vec3 viewDirection = normalize(camera.position - inFragPosition);
    // The direction of reflection of light - the inverse of the light to the surface.
    vec3 reflectDirection = reflect(-lightDir, norm);
    // The specular is basically like diffuse, plus an extra "specular strength". There's also a coarseness, which
    // determines the size of the "dot" created from the shading itself.
    float specStrength = 0.5;
    int specCoarseness = 32;
    float spec = pow(max(dot(viewDirection, reflectDirection), 0.0), specCoarseness);
    vec3 specular = specStrength * spec * lightColor;

    vec3 result = (ambient + diffuse + specular) * objectColor;

    f_color = vec4(result, 1.0);
}
