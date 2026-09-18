//! Python bindings for rapidagent-compiler

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::PathBuf;

/// Compile a .rag file and return JSON result
#[pyfunction]
fn compile_rag(source_path: &str, format: &str) -> PyResult<String> {
    let path = PathBuf::from(source_path);
    
    // Read source
    let source = std::fs::read_to_string(&path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    
    // Parse rag format
    let rag_artifact = rapidagent_core::parse_rag(&source)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    
    // Convert to PromptIR for rendering
    let ir = rag_artifact.to_prompt_ir();
    
    // Render based on format
    let output = match format {
        "xml" => rapidagent_renderer::render_xml(&ir),
        "markdown" | "md" => rapidagent_renderer::render_markdown(&ir),
        "json" => rapidagent_renderer::render_json(&ir)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?,
        _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Unsupported format: {format}. Use 'xml', 'markdown', or 'json'.")
        )),
    };
    
    Ok(output)
}

/// Get metadata about a compiled agent
#[pyfunction]
fn get_metadata(source_path: &str) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        let path = PathBuf::from(source_path);
        
        let source = std::fs::read_to_string(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        
        let rag_artifact = rapidagent_core::parse_rag(&source)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        
        let ir = rag_artifact.to_prompt_ir();
        
        let dict = PyDict::new(py);
        dict.set_item("name", &ir.name)?;
        dict.set_item("version", ir.version.to_string())?;
        dict.set_item("section_count", ir.sections.len())?;
        
        Ok(dict.into())
    })
}

/// Scan for security issues in a .rag file
#[pyfunction]
fn security_scan(source_path: &str) -> PyResult<Vec<String>> {
    let path = PathBuf::from(source_path);
    let source = std::fs::read_to_string(&path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    
    let rag_artifact = rapidagent_core::parse_rag(&source)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    
    let ir = rag_artifact.to_prompt_ir();
    let warnings = rapidagent_security::scan_for_injection(&ir);
    
    Ok(warnings.iter()
        .map(|w| format!("[{}] {}: {}", w.severity, w.section, w.message))
        .collect())
}

/// Python module definition
#[pymodule]
fn syspro_compiler(m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile_rag, m)?)?;
    m.add_function(wrap_pyfunction!(get_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(security_scan, m)?)?;
    Ok(())
}
