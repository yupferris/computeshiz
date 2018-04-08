extern crate gl;
extern crate glutin;

use gl::types::*;
use glutin::*;

use std::ffi::CString;
use std::mem;
use std::ptr;

fn main() {
    if let Err(e) = do_it() {
        println!("Awww, shoot! {}", e);
    }
}

fn do_it() -> Result<(), String> {
    let context = HeadlessRendererBuilder::new(0, 0)
        .with_gl(GlRequest::Specific(Api::OpenGl, (4, 3)))
        .build_strict()
        .map_err(|e| format!("Could not create headless context: {}", e))?;

    unsafe {
        // Set up GL
        context.make_current().map_err(|e| format!("Could not make context current: {}", e))?;

        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

        // Create shader
        //  TODO: Check errors etc
        let shader = gl::CreateShader(gl::COMPUTE_SHADER);
        let src = "

#version 430

layout(local_size_x = 4) in;

layout(std430, binding = 0) buffer resultBuffer
{
    uint result[];
};

void main()
{
    result[gl_LocalInvocationIndex] = result[gl_LocalInvocationIndex] + gl_LocalInvocationIndex;
}

        ";
        let c_src = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_src.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let program = gl::CreateProgram();
        gl::AttachShader(program, shader);
        gl::LinkProgram(program);

        // Create result buffer
        let mut ssbo: GLuint = 0;
        gl::GenBuffers(1, &mut ssbo as *mut _);
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo);
        let initial_values = [32u32; 4];
        gl::BufferData(gl::SHADER_STORAGE_BUFFER, mem::size_of::<[u32; 4]>() as _, &initial_values as *const [u32; 4] as *const _, gl::DYNAMIC_COPY);
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);

        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, ssbo);

        gl::UseProgram(program);
        gl::DispatchCompute(1, 1, 1);

        gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);

        // Read result
        let res = {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo);
            let ptr = gl::MapBuffer(gl::SHADER_STORAGE_BUFFER, gl::READ_ONLY) as *const [u32; 4];
            let res = *ptr;
            gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
            res
        };

        println!("Result: {:?}", res);

        // TODO: Cleanup etc   
    }

    Ok(())
}
