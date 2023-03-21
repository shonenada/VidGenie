#version 450 core
layout (location = 0) in float inTexIdx;
layout (location = 1) in vec2 position;
layout (location = 2) in vec2 verTexCoord;

out vec2 texCoord;
out float texIdxf;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    texCoord = verTexCoord;
    texIdxf = inTexIdx;
}