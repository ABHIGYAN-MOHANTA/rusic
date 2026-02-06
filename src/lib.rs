use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlBuffer, WebGlUniformLocation};

// Number of bars - use full resolution
const NUM_BARS: usize = 128;

// Store WebGL state
thread_local! {
    static STATE: RefCell<Option<VisualizerState>> = RefCell::new(None);
}

struct VisualizerState {
    gl: WebGl2RenderingContext,
    program: WebGlProgram,
    vertex_buffer: WebGlBuffer,
    resolution_loc: WebGlUniformLocation,
    canvas_width: f32,
    canvas_height: f32,
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    let canvas = document
        .get_element_by_id("canvas")
        .ok_or("No canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    
    let gl = canvas
        .get_context("webgl2")?
        .ok_or("No WebGL2")?
        .dyn_into::<WebGl2RenderingContext>()?;
    
    // Vertex shader
    let vert_src = r#"#version 300 es
        precision highp float;
        
        in vec2 a_position;
        in float a_value;
        in float a_index;
        
        uniform vec2 u_resolution;
        
        out float v_value;
        out float v_index;
        out vec2 v_pos;
        
        void main() {
            v_value = a_value;
            v_index = a_index;
            v_pos = a_position;
            
            vec2 clipSpace = (a_position / u_resolution) * 2.0 - 1.0;
            gl_Position = vec4(clipSpace * vec2(1, -1), 0, 1);
        }
    "#;
    
    // Fragment shader - Neon pill bars
    let frag_src = r#"#version 300 es
        precision highp float;
        
        in float v_value;
        in float v_index;
        in vec2 v_pos;
        
        out vec4 fragColor;
        
        void main() {
            // Gradient: Cyan -> Indigo -> Pink
            vec3 c1 = vec3(0.0, 0.84, 1.0);    // Cyan #00d7ff
            vec3 c2 = vec3(0.39, 0.4, 0.95);   // Indigo #6366f1
            vec3 c3 = vec3(1.0, 0.18, 0.58);   // Pink #ff2d95
            
            // Interpolate color based on bar index (left to right)
            float t = v_index / 128.0;
            vec3 color = mix(c1, c2, smoothstep(0.0, 0.5, t));
            color = mix(color, c3, smoothstep(0.5, 1.0, t));
            
            // Add glow intensity based on volume
            float glow = 0.5 + v_value * 0.5;
            color *= glow; // Bloom effect
            
            // Vertical fade for softness at tips
            // Assuming bars are centered at Y, we can cheat by just using solid color
            // as the shape is defined by geometry.
            
            // Slight transparency for glass feel
            float alpha = 0.9 + v_value * 0.1;
            
            fragColor = vec4(color, alpha);
        }
    "#;
    
    let vert_shader = compile_shader(&gl, WebGl2RenderingContext::VERTEX_SHADER, vert_src)?;
    let frag_shader = compile_shader(&gl, WebGl2RenderingContext::FRAGMENT_SHADER, frag_src)?;
    let program = link_program(&gl, &vert_shader, &frag_shader)?;
    
    gl.use_program(Some(&program));
    
    let resolution_loc = gl.get_uniform_location(&program, "u_resolution")
        .ok_or("No resolution uniform")?;
    
    let vertex_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
    
    let pos_loc = gl.get_attrib_location(&program, "a_position") as u32;
    let val_loc = gl.get_attrib_location(&program, "a_value") as u32;
    let idx_loc = gl.get_attrib_location(&program, "a_index") as u32;
    
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    
    let stride = 4 * 4;
    gl.vertex_attrib_pointer_with_i32(pos_loc, 2, WebGl2RenderingContext::FLOAT, false, stride, 0);
    gl.vertex_attrib_pointer_with_i32(val_loc, 1, WebGl2RenderingContext::FLOAT, false, stride, 8);
    gl.vertex_attrib_pointer_with_i32(idx_loc, 1, WebGl2RenderingContext::FLOAT, false, stride, 12);
    
    gl.enable_vertex_attrib_array(pos_loc);
    gl.enable_vertex_attrib_array(val_loc);
    gl.enable_vertex_attrib_array(idx_loc);
    
    gl.enable(WebGl2RenderingContext::BLEND);
    gl.blend_func(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA);
    
    let width = canvas.width() as f32;
    let height = canvas.height() as f32;
    
    STATE.with(|s| {
        *s.borrow_mut() = Some(VisualizerState {
            gl,
            program,
            vertex_buffer,
            resolution_loc,
            canvas_width: width,
            canvas_height: height,
        });
    });
    
    Ok(())
}

#[wasm_bindgen]
pub fn render_frame(frequency_data: &[u8], _time_data: &[u8]) {
    STATE.with(|s| {
        let mut state_ref = s.borrow_mut();
        if let Some(state) = state_ref.as_mut() {
            render_linear_visualizer(state, frequency_data);
        }
    });
}

#[wasm_bindgen]
pub fn update_canvas_size(width: f32, height: f32) {
    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            state.canvas_width = width;
            state.canvas_height = height;
            state.gl.viewport(0, 0, width as i32, height as i32);
        }
    });
}

fn render_linear_visualizer(state: &mut VisualizerState, frequency_data: &[u8]) {
    let gl = &state.gl;
    
    // Clear
    gl.clear_color(0.0, 0.0, 0.0, 0.0);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    
    // Set uniforms
    gl.uniform2f(Some(&state.resolution_loc), state.canvas_width, state.canvas_height);
    
    let cy = state.canvas_height / 2.0;
    
    // We want the spectrum to span the width, with some padding
    let padding_x = state.canvas_width * 0.05;
    let total_width = state.canvas_width - (2.0 * padding_x);
    let bar_spacing = total_width / NUM_BARS as f32;
    let bar_width = bar_spacing * 0.7; // 70% bar, 30% gap
    
    let max_height = state.canvas_height * 0.35; // Leave some room
    
    let mut vertices: Vec<f32> = Vec::with_capacity(NUM_BARS * 6 * 4);
    
    for i in 0..NUM_BARS {
        // Frequency mapping
        // We often want to skip the very first few low bins as they can be DC offset or hum
        // and maybe limit the top end.
        // But for simplicity, let's just map 1:1 if we count 128 bins.
        let freq_idx = i.min(frequency_data.len() - 1);
        let value = frequency_data[freq_idx] as f32 / 255.0;
        
        let smoothed_value = value.powf(0.85); // Gamma correction
        
        // Calculate X position
        let x_center = padding_x + (i as f32 * bar_spacing) + (bar_spacing / 2.0);
        
        // Bar half-height (mirrored)
        let h = 4.0 + (smoothed_value * max_height); // Minimum 4px height
        
        // Coordinates for a centered rounded rect (simulated by simple rect)
        let x1 = x_center - bar_width / 2.0;
        let x2 = x_center + bar_width / 2.0;
        let y_top = cy - h;
        let y_bottom = cy + h;
        
        let idx = i as f32;
        
        // Triangle 1
        vertices.extend_from_slice(&[x1, y_top, smoothed_value, idx]);
        vertices.extend_from_slice(&[x2, y_top, smoothed_value, idx]);
        vertices.extend_from_slice(&[x1, y_bottom, smoothed_value, idx]);
        
        // Triangle 2
        vertices.extend_from_slice(&[x2, y_top, smoothed_value, idx]);
        vertices.extend_from_slice(&[x2, y_bottom, smoothed_value, idx]);
        vertices.extend_from_slice(&[x1, y_bottom, smoothed_value, idx]);
    }
    
    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }
    
    gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, (NUM_BARS * 6) as i32);
}

fn compile_shader(gl: &WebGl2RenderingContext, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    let shader = gl.create_shader(shader_type).ok_or("Cannot create shader")?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    
    if gl.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl.get_shader_info_log(&shader).unwrap_or_default())
    }
}

fn link_program(gl: &WebGl2RenderingContext, vert: &WebGlShader, frag: &WebGlShader) -> Result<WebGlProgram, String> {
    let program = gl.create_program().ok_or("Cannot create program")?;
    gl.attach_shader(&program, vert);
    gl.attach_shader(&program, frag);
    gl.link_program(&program);
    
    if gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl.get_program_info_log(&program).unwrap_or_default())
    }
}
