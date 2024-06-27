shader_type canvas_item;

// list of humidities for the areas visible on screen
uniform vec2 position;
uniform float max_humidity;
uniform float[100] humidities;
uniform vec2[100] humidity_grid_positions;
uniform vec2 grid_size;
uniform vec2 total_size;
uniform sampler2D noise;

void vertex() {
	// Called for every vertex the material is visible
}

float get_humidity_on_grid_position(ivec2 grid_position){
	for(int idx=0; idx<humidity_grid_positions.length(); idx++){
		if(ivec2(humidity_grid_positions[idx])==grid_position){
			return humidities[idx]/max_humidity;
		}
	}
	return 0.0;
}

vec2 get_real_pos_of_grid(ivec2 grid_position){
	return grid_size*vec2(grid_position);
}

float get_input_to_position(ivec2 input_grid,vec2 to_position){
	vec2 relative_pos_of_grid=get_real_pos_of_grid(input_grid)+grid_size/2.0;
	float distance_to_point=distance(relative_pos_of_grid,to_position);
	float multiplier=clamp(0.8-distance_to_point/grid_size.x,0.0,1.0);
	return get_humidity_on_grid_position(input_grid)*multiplier;
}

void fragment() {
	vec2 real_position=UV * total_size + position;
	ivec2 grid_position=ivec2(floor(real_position/grid_size));
	COLOR=texture(noise,UV);
	float humidity=0.0;
	humidity+=get_input_to_position(grid_position,real_position);
	humidity+=get_input_to_position(grid_position+ivec2(1,0),real_position);
	humidity+=get_input_to_position(grid_position+ivec2(-1,0),real_position);
	humidity+=get_input_to_position(grid_position+ivec2(0,-1),real_position);
	humidity+=get_input_to_position(grid_position+ivec2(0,1),real_position);
	//corners
	humidity+=get_input_to_position(grid_position+ivec2(1,1),real_position);
	humidity+=get_input_to_position(grid_position+ivec2(-1,1),real_position);
	humidity+=get_input_to_position(grid_position+ivec2(1,-1),real_position);
	humidity+=get_input_to_position(grid_position+ivec2(-1,-1),real_position);
	COLOR.a=humidity;
}
