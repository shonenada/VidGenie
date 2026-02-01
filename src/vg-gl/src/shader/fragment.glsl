#version 330 core
out vec4 FragColor;

in vec2 texCoord;

uniform sampler2D uTexture;
uniform float uAlpha;

void main() {
    vec4 color = texture(uTexture, texCoord);
    FragColor = vec4(color.rgb, color.a * uAlpha);
}
