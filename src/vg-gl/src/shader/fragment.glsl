#version 450 core
out vec4 FragColor;

in vec2 texCoord;
in float texIdxf;

uniform sampler2D textures[32];
uniform float uAlpha;

void main() {
    int texIdx = int(texIdxf);
    vec4 color = texture(textures[texIdx], texCoord);
    FragColor = vec4(color.rgb, color.a * uAlpha);
}
