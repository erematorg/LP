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

void fragment() {
	vec2 real_relative_position=UV * total_size;
	ivec2 grid_position=ivec2(floor(real_relative_position/grid_size));
	vec2 position_between_grid = real_relative_position-grid_size*vec2(grid_position);
	vec2 advancement_in_grid = position_between_grid/grid_size;
	COLOR=texture(noise,UV);
	float humidity=get_humidity_on_grid_position(grid_position);
	if (advancement_in_grid.y>0.5){
		float down_humidity = get_humidity_on_grid_position(grid_position+ivec2(0,1));
		float advancement_multiplier=(advancement_in_grid.y-0.5)/0.5;
		humidity+=(down_humidity-humidity)*advancement_multiplier;
	}
	COLOR.a=humidity;
}