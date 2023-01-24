layout(set = 0, binding = 0) GlobalUbo {
	mat4 view;
	mat4 proj;
	mat4 view_proj;
	mat4 reserved_0;
	sampler2D global_tex;
} global;

layout(set = 1, binding = 0) LocalUbo {
	mat4 model;
} local;


