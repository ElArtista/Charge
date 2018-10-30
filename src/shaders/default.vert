#version 300 es
in vec3 vpos;
in vec3 vnrm;
in vec2 vuv0;

out vec2 texcoord;
out vec3 normal;
out vec3 ws_pos;

uniform mat4 model;
uniform mat4 mvp;
uniform mat3 nmm;

void main()
{
    texcoord = vuv0;
    ws_pos = (model * vec4(vpos, 1.0)).xyz;
    normal = nmm * vnrm;
    gl_Position = mvp * vec4(vpos, 1.0);
}
