#version 460

// Per-vertex information - Vertex3 struct
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

// Per-instance information - GpuInstance struct
layout(location = 2) in vec4 transform_col1;
layout(location = 3) in vec4 transform_col2;
layout(location = 4) in vec4 transform_col3;
layout(location = 5) in vec4 transform_col4;

// Per-frame information - CameraUniform struct
layout(set = 0, binding = 0) uniform CameraUniform {
    mat4 view;
    mat4 projection;
} camera;

void main() {
    // We then multiply each vertex by its transform to find the correct location in the scene.
    mat4 transform = mat4(transform_col1, transform_col2, transform_col3, transform_col4);

    mat4 mvp = camera.projection * camera.view * transform;

    gl_Position = mvp * vec4(position, 1.0f);
}
