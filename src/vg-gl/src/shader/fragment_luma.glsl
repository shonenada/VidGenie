#version 330 core
out vec4 FragColor;

in vec2 texCoord;

uniform sampler2D uTexture;
uniform sampler2D uLumaMask;
uniform float uAlpha;

void main() {
    vec4 color = texture(uTexture, texCoord);
    vec4 mask = texture(uLumaMask, texCoord);
    float luminance = dot(mask.rgb, vec3(0.299, 0.587, 0.114));
    float maskAlpha = luminance * mask.a;
    FragColor = vec4(color.rgb, color.a * uAlpha * maskAlpha);
}
