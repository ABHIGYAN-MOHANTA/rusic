use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, console};

// Store the WebGL context globally for render_frame to use
static mut GL_CONTEXT: Option<WebGl2RenderingContext> = None;
static mut PROGRAM: Option<WebGlProgram> = None;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Access the document and canvas
    let window = web_sys::window().ok_or("No global window found")?;
    let document = window.document().ok_or("No document found")?;
    let canvas = document
        .get_element_by_id("canvas")
        .ok_or("No canvas found")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    
    // Get the WebGL rendering context
    let gl = canvas
        .get_context("webgl2")?
        .ok_or("Failed to get WebGL2 context")?
        .dyn_into::<WebGl2RenderingContext>()?;
    
    // Initialize shaders
    let vertex_shader = compile_shader(
        &gl,
        WebGl2RenderingContext::VERTEX_SHADER,
        r#"
        attribute vec4 position;
        void main() {
            gl_Position = position;
        }
        "#,
    )?;
    let fragment_shader = compile_shader(
        &gl,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r#"
        precision mediump float;
        void main() {
            gl_FragColor = vec4(0.0, 1.0, 0.0, 1.0);
        }
        "#,
    )?;
    let program = link_program(&gl, &vertex_shader, &fragment_shader)?;
    gl.use_program(Some(&program));
    
    // Set up the vertices
    let vertices: [f32; 6] = [
        0.0,  0.5,  // Top vertex
       -0.5, -0.5,  // Bottom left vertex
        0.5, -0.5,  // Bottom right vertex
    ];
    let buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    unsafe {
        let vertices_array = js_sys::Float32Array::view(&vertices);
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vertices_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }
    
    // Link the position attribute in the vertex shader
    let position = gl.get_attrib_location(&program, "position") as u32;
    gl.vertex_attrib_pointer_with_i32(position, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(position);
    
    // Store GL context and program globally for render_frame
    unsafe {
        GL_CONTEXT = Some(gl.clone());
        PROGRAM = Some(program);
    }
    
    // Initial draw
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);
    
    Ok(())
}

/// Called every animation frame with audio frequency data
/// frequency_data: Uint8Array of 128 values (0-255) representing frequency bins from bass to treble
/// time_data: Uint8Array of 128 values (0-255) representing waveform data
#[wasm_bindgen]
pub fn render_frame(frequency_data: &[u8], time_data: &[u8]) {
    // Log audio data info (you can remove this once you start building your visualizer)
    let freq_len = frequency_data.len();
    let time_len = time_data.len();
    
    // Calculate some useful metrics from the frequency data
    let bass_avg: u32 = frequency_data[0..16].iter().map(|&x| x as u32).sum::<u32>() / 16;
    let mid_avg: u32 = frequency_data[16..64].iter().map(|&x| x as u32).sum::<u32>() / 48;
    let treble_avg: u32 = frequency_data[64..128].iter().map(|&x| x as u32).sum::<u32>() / 64;
    let overall_avg: u32 = frequency_data.iter().map(|&x| x as u32).sum::<u32>() / freq_len as u32;
    
    // Log every ~60 frames to avoid spam (check if bass_avg changed significantly)
    static mut LAST_LOG: u32 = 0;
    static mut FRAME_COUNT: u32 = 0;
    unsafe {
        FRAME_COUNT += 1;
        if FRAME_COUNT % 30 == 0 {
            console::log_1(&format!(
                "🎵 Audio Data | Freq bins: {} | Time bins: {} | Bass: {} | Mid: {} | Treble: {} | Overall: {}",
                freq_len, time_len, bass_avg, mid_avg, treble_avg, overall_avg
            ).into());
        }
    }
    
    // TODO: Your WebGL visualization code goes here!
    // You have access to:
    // - frequency_data: 128 frequency bins (0-255), index 0 = bass, index 127 = treble
    // - time_data: 128 waveform samples (0-255), centered at 128
    // - bass_avg, mid_avg, treble_avg, overall_avg: pre-calculated averages
    //
    // Example: Use bass_avg to pulse the size of shapes, treble_avg for color intensity, etc.
}

/// Get audio analysis info as a string (for debugging in JS console)
#[wasm_bindgen]
pub fn get_audio_info(frequency_data: &[u8]) -> String {
    let bass: u32 = frequency_data[0..16].iter().map(|&x| x as u32).sum::<u32>() / 16;
    let mid: u32 = frequency_data[16..64].iter().map(|&x| x as u32).sum::<u32>() / 48;
    let treble: u32 = frequency_data[64..128].iter().map(|&x| x as u32).sum::<u32>() / 64;
    
    format!("Bass: {} | Mid: {} | Treble: {}", bass, mid, treble)
}

fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or("Unable to create shader object")?;
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

fn link_program(
    gl: &WebGl2RenderingContext,
    vertex_shader: &WebGlShader,
    fragment_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or("Unable to create shader program")?;
    gl.attach_shader(&program, vertex_shader);
    gl.attach_shader(&program, fragment_shader);
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
