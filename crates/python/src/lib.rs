//! Python bindings for rw-syspro-compiler

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::PathBuf;

/// Compile a .spc file and return the result
#[pyfunction]
fn compile(source_path: &str, format: &str) -> PyResult<String> {
    let path = PathBuf::from(source_path);
    
    // Read source
    let source = std::fs::read_to_string(&path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    
    // Tokenize
    let mut lexer = rapidagent_core::Lexer::new(&source);
    let tokens = lexer.tokenize();
    
    // Parse
    let ir = rapidagent_core::parse_tokens(&tokens)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
    
    // Optimize
    rapidagent_core::optimize(&mut ir.clone());
    
    // Render
    let output = match format {
        "xml" => rapidagent_renderer::render_xml(&ir),
        "markdown" | "md" => rapidagent_renderer::render_markdown(&ir),
        "json" => rapidagent_renderer::render_json(&ir),
        _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Unsupported format: {}. Use 'xml', 'markdown', or 'json'."))
        ),
    };
    
    Ok(output)
}

/// Get metadata about a compiled prompt
#[pyfunction]
fn get_metadata(source_path: &str) -> PyResult<PyObject> {
    let python = Python::get_thread_interpreter();
    let path = PathBuf::from(source_path);
    
    let source = std::fs::read_to_string(&path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    
    let mut lexer = rapidagent_core::Lexer::new(&source);
    let tokens = lexer.tokenize();
    let ir = rapidagent_core::parse_tokens(&tokens)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    
    let dict = PyDict::new(python);
    dict.set_item("name", &ir.name)?;
    dict.set_item("version", ir.version.to_string())?;
    dict.set_item("hash", ir.content_hash())?;
    dict.set_item("section_count", ir.sections.len())?;
    
    let sections: Vec<PyObject> = ir.sections.iter()
        .map(|s| {
            let sdict = PyDict::new(python);
            sdict.set_item("name", &s.name).unwrap();
            sdict.set_item("priority", s.priority.to_string()).unwrap();
            sdict.set_item("weight", s.priority.weight()).unwrap();
            sdict.to_object(python)
        })
        .collect();
    dict.set_item("sections", sections)?;
    
    Ok(dict.to_object(python))
}

/// Scan for security issues
#[pyfunction]
fn security_scan(source_path: &str) -> PyResult<Vec<String>> {
    let path = PathBuf::from(source_path);
    let source = std::fs::read_to_string(&path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    
    let mut lexer = rapidagent_core::Lexer::new(&source);
    let tokens = lexer.tokenize();
    let ir = rapidagent_core::parse_tokens(&tokens)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    
    let warnings = rapidagent_security::scan_for_injection(&ir);
    
    Ok(warnings.iter()
        .map(|w| format!("[{}] {}: {}", w.severity, w.section, w.message))
        .collect())
}

/// Python module definition
#[pymodule]
fn syspro_compiler(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(get_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(security_scan, m)?)?;
    Ok(())
}
