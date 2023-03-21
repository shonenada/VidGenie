#version 450 core
out vec4 FragColor;

in vec2 texCoord;
in float texIdxf;

uniform sampler2D textures[32];

void main() {
    int texIdx = int(texIdxf);
    FragColor = texture(textures[texIdx], texCoord);
}