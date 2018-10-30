#version 300 es
#ifdef GL_ES
precision mediump float;
#endif
out vec4 fcolor;

in vec2 texcoord;
in vec3 normal;
in vec3 ws_pos;

uniform sampler2D tex;
uniform vec3 light_pos;

const vec3 light_color = vec3(1.0);

void main()
{
    vec3 base_color = texture(tex, texcoord).rgb;
    vec3 N = normalize(normal);
    vec3 L = normalize(light_pos - ws_pos);
    float kD = max(dot(N, L), 0.0);
    vec3 color = kD * base_color * light_color;
    fcolor = vec4(color, 1.0);
}
