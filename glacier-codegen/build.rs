use std::collections::HashSet;
use std::{fs, io};
use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::Path;
use std::str::FromStr;
use regex::Regex;

macro_rules! warn {
    ($($tokens: tt)*) => {
        println!("cargo::warning={}", format!($($tokens)*))
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs"); //prevent this file from running every time

    generate_bindings(
        "../extern/ZHMTools/Libraries/ResourceLib/Src/Generated/HM2016/ZHMGen.h",
        "hm2016_bindings"
    ).unwrap();

    generate_bindings(
        "../extern/ZHMTools/Libraries/ResourceLib/Src/Generated/HM2/ZHMGen.h",
        "hm2_bindings"
    ).unwrap();

    generate_bindings(
        "../extern/ZHMTools/Libraries/ResourceLib/Src/Generated/HM3/ZHMGen.h",
        "hm3_bindings"
    ).unwrap();
}


pub fn generate_bindings<P: AsRef<Path>>(header_path: P, module_name: &str) -> io::Result<()> {

    // Read the entire C++ header file.
    let header_content = fs::read_to_string(header_path)
        .expect("Failed to read header file");

    let output_dir = format!("src/{}", module_name);
    fs::create_dir_all(&output_dir)?;

    let enum_path = format!("{}/enums.rs", &output_dir);

    let mut enums_buffer = BufWriter::new(File::create(enum_path)?);

    let properties_path = format!("{}/properties.rs", &output_dir);
    let mut properties_buffer = BufWriter::new(File::create(properties_path)?);
    
    let mod_path = format!("{}/mod.rs", &output_dir);
    let mut mod_buffer = BufWriter::new(File::create(mod_path)?);
    writeln!(mod_buffer, "pub mod enums;")?;
    writeln!(mod_buffer, "pub mod properties;")?;
    
    // Generate code for enums.
    generate_enums(&header_content, &mut enums_buffer)?;

    // Generate code for classes (properties).
    generate_properties(&header_content, &mut properties_buffer, &module_name)?;
    Ok(())
}

fn write_includes<W: io::Write>(buffer: &mut W) -> io::Result<()>{
    writeln!(buffer, "use crate::glacier_types::*;")?;
    writeln!(buffer, "use serde::{{ Serialize, Deserialize }};")?;
    writeln!(buffer, "use std::collections::HashMap;")?;
    writeln!(buffer, "use serde_big_array::BigArray;")?;
    writeln!(buffer, "")?;
    Ok(())
}

/// Extracts and generates Rust code for all C++ enum classes in the header.
fn generate_enums<W: io::Write>(header_content: &str, buffer: &mut W) -> io::Result<()> {
    let enum_regex = Regex::new(r"enum class\s*?\s*(\w+)").unwrap();
    let enum_value_regex = Regex::new(r"(?m)^\s*([A-Za-z_]\w*)(?:\s*=\s*([^,]+))?\s*(?:,\s*)?(?:\/\/.*)?$").unwrap();

    let mut written_symbols = HashSet::new();

    write_includes(buffer)?;

    for caps in enum_regex.captures_iter(header_content) {
        let original_enum_name = &caps[1];
        let rust_enum_name = heck::AsUpperCamelCase(&caps[1]).to_string();

        // Skip if we've already processed this symbol (avoid duplicates).
        if written_symbols.contains(&rust_enum_name) {
            continue;
        }

        // Find the block of text between '{' and the matching '}'.
        if let Some(start_idx) = header_content[caps.get(0).unwrap().end()..].find('{') {
            let offset = caps.get(0).unwrap().end() + start_idx + 1;
            let mut brace_count = 1;
            let mut end_pos = offset;

            for (i, c) in header_content[offset..].chars().enumerate() {
                match c {
                    '{' => brace_count += 1,
                    '}' => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            end_pos = offset + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let enum_body = &header_content[offset..end_pos];
            let mut fields = Vec::new();
            let mut existing_values = HashSet::new();

            for fcap in enum_value_regex.captures_iter(enum_body) {
                let raw_variant_name = fcap[1].trim();
                let mut raw_value = fcap
                    .get(2)
                    .map(|m| m.as_str().to_owned());

                // If a numeric value has repeated usage, prefix with '-' to avoid reuse:
                if let Some(val_str) = &raw_value {
                    if existing_values.contains(val_str) {
                        raw_value = Some(format!("-{}", val_str));
                    } else {
                        existing_values.insert(val_str.to_string());
                    }
                }

                let rust_field_name = heck::AsUpperCamelCase(raw_variant_name.trim()).to_string();
                fields.push((raw_variant_name.to_string(), rust_field_name, raw_value));
            }

            // Remove common prefix if it exists (e.g., "E_" from all variants).
            remove_common_prefix_from_enum_variants(&mut fields);


            writeln!(buffer, "#[derive(Debug, Serialize, Deserialize)]")?;
            writeln!(buffer, "#[serde(rename = \"{}\")]", original_enum_name)?;
            writeln!(buffer, "pub enum {} {{", rust_enum_name)?;
            for (original_field, rust_field, val_str) in fields {
                writeln!(buffer, "\t#[serde(rename = \"{}\")]", original_field)?;
                if val_str.as_ref().is_some_and(|v| !v.is_empty()) {
                    writeln!(buffer, "\t{} = {},", rust_field, val_str.unwrap())?;
                } else {
                    writeln!(buffer, "\t{},", rust_field)?;
                }
            }
            writeln!(buffer, "}}")?;
            writeln!(buffer, "")?;

            // Register ZVariant trait using the original name or safe modifications
            let variant_handle = if original_enum_name.contains("eParticleEmitterBoxEntity") {
                original_enum_name.to_string()
            } else {
                replace_last(original_enum_name, "_", ".")
            };

            writeln!(buffer, "#[typetag::serde(name = \"{}\")]", variant_handle)?;
            writeln!(buffer, "impl ZVariant for {} {{}}", rust_enum_name)?;

            writeln!(buffer, "#[typetag::serde(name = \"TArray<{}>\")]", variant_handle)?;
            writeln!(buffer, "impl ZVariant for TArray<{}> {{}}", rust_enum_name)?;
        }

        written_symbols.insert(rust_enum_name);
    }
    Ok(())
}

fn generate_properties<W: io::Write>(header_content: &str, buffer: &mut W, module_name: &str) -> io::Result<()>{
    let class_regex = Regex::new(r"(?:^|\W)(enum )?class\s*(?:/\*\s*alignas\(\d+\)\s*\*/)?\s*(\w+)").unwrap();
    let field_regex = Regex::new(r"(?m)^\s*(?:[a-zA-Z0-9_]+::)?([<>\w]+(?:<[^>]+>)?)\s+(\w+);\s*(?:\/\/.*)?$").unwrap();
    
    write_includes(buffer)?;
    writeln!(buffer, "use crate::{}::enums::*;", module_name)?;

    for caps in class_regex.captures_iter(header_content) {
        if caps.get(1).is_some_and(|cap| cap.as_str() == "enum "){ //if the enum prefix was captured
            continue;
        }

        let original_class_name = &caps[2];
        let rust_class_name = heck::AsUpperCamelCase(&caps[2]).to_string();

        // Find the block of text between '{' and the matching '}'.
        if let Some(start_idx) = header_content[caps.get(0).unwrap().end()..].find('{') {
            let offset = caps.get(0).unwrap().end() + start_idx + 1;
            let mut brace_count = 1;
            let mut end_pos = offset;

            for (i, c) in header_content[offset..].chars().enumerate() {
                match c {
                    '{' => brace_count += 1,
                    '}' => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            end_pos = offset + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let class_body = &header_content[offset..end_pos];
            let mut fields = Vec::new();

            for fcap in field_regex.captures_iter(class_body) {
                let mut raw_type = fcap[1].trim();
                let raw_name = fcap[2].trim().to_string();

                // Special case
                if rust_class_name == "SEntityTemplateProperty" && raw_name == "nPropertyID" {
                    raw_type = "EntityTemplatePropertyId";
                }

                let mut rust_field_name = map_hungarian(&raw_name);

                // Convert the type into a Rust-friendly version
                let rust_type = map_cpp_type_to_rust(raw_type);
                if rust_type == "bool" && !rust_field_name.starts_with("is_") {
                    rust_field_name = format!("is_{}", rust_field_name);
                }

                fields.push((rust_field_name, raw_name, rust_type));
            }

            
            writeln!(buffer, "#[derive(Debug, Serialize, Deserialize)]")?;
            writeln!(buffer, "#[serde(rename = \"{}\")]", original_class_name)?;
            writeln!(buffer, "pub struct {} {{", rust_class_name)?;

            for (rust_field_name, orig_field_name, rust_type) in fields {
                writeln!(buffer,"\t#[serde(rename = \"{}\")]", orig_field_name)?;

                // If the field is a fixed array and is large, apply the BigArray attribute:
                let fixed_arr_re = Regex::new(r"^\s*\[\s*\w+\s*;\s*(\d+)\s*\]\s*$").unwrap();
                if let Some(caps) = fixed_arr_re.captures(&rust_type) {
                    if let Ok(size) = caps[1].parse::<usize>() {
                        if size > 32 {
                            writeln!(buffer, "\t#[serde(with = \"serde_big_array::BigArray\")]")?;
                        }
                    }
                }
                writeln!(buffer, "\tpub {}: {},", rust_field_name, rust_type)?;
            }

            writeln!(buffer, "}}")?;
            writeln!(buffer, "")?;

            // Provide a ZVariant trait
            let variant_handle = replace_last(original_class_name, "_", ".");
            writeln!(buffer, "#[typetag::serde(name = \"{}\")]", variant_handle)?;
            writeln!(buffer, "impl ZVariant for {} {{}}", rust_class_name)?;

            writeln!(buffer, "#[typetag::serde(name = \"TArray<{}>\")]", variant_handle)?;
            writeln!(buffer, "impl ZVariant for TArray<{}> {{}}", rust_class_name)?;
        }
    }

    Ok(())
}

/// Minimal helper to map some known C++ type strings to Rust equivalents.
fn map_cpp_type_to_rust(cpp_type: &str) -> String {
    let array_tarray_re = Regex::new(r"^TArray<(.+)>$").unwrap();
    let fixed_array_re = Regex::new(r"^TFixedArray<([A-Za-z0-9_]+),\s*(\d+)>$").unwrap();
    let map_re = Regex::new(r"^TMap<([A-Za-z0-9_]+),\s*([A-Za-z0-9_]+)>$").unwrap();

    match cpp_type {
        "float32" | "float" => "f32".to_owned(),
        "float64" | "double" => "f64".to_owned(),
        "int64" => "i64".to_owned(),
        "uint64" => "u64".to_owned(),
        "int32" | "int" => "i32".to_owned(),
        "uint32" | "unsigned" => "u32".to_owned(),
        "int16" => "i16".to_owned(),
        "uint16" => "u16".to_owned(),
        "int8" => "i8".to_owned(),
        "uint8" => "u8".to_owned(),
        "bool" => "bool".to_owned(),
        "char" => "char".to_owned(),
        "ZVariant" => "Box<dyn ZVariant>".to_owned(),
        _ => {
            if let Some(caps) = array_tarray_re.captures(cpp_type) {
                let inner = map_cpp_type_to_rust(&caps[1]);
                return format!("Vec<{}>", inner);
            }
            if let Some(caps) = fixed_array_re.captures(cpp_type) {
                let inner = map_cpp_type_to_rust(&caps[1]);
                let len = &caps[2];
                return format!("[{}; {}]", inner, len);
            }
            if let Some(caps) = map_re.captures(cpp_type) {
                let key = map_cpp_type_to_rust(&caps[1]);
                let value = map_cpp_type_to_rust(&caps[2]);
                return format!("std::collections::HashMap<{}, {}>", key, value);
            }
            // Fallback:
            let res = cpp_type.replace("::", "_");
            heck::AsUpperCamelCase(res).to_string()
        }
    }
}

fn map_hungarian(var_name: &str) -> String {
    // Hungarian notations like "m_" or "mValue"
    let hungarian_re3 = Regex::new(r"^[a-zA-Z]_").unwrap();
    let trimmed = if hungarian_re3.is_match(&var_name) {
        hungarian_re3.replace(var_name, "").to_string()
    } else {
        var_name.to_string()
    };
    let candidate = heck::AsSnakeCase(trimmed).to_string();
    // Keywords and collisions
    match candidate.as_str() {
        "type" => "type_".to_string(),
        "move" => "move_".to_string(),
        _ => candidate,
    }
}

fn remove_common_prefix_from_enum_variants(fields: &mut Vec<(String, String, Option<String>)>) {
    if fields.is_empty() {
        return;
    }

    let first_raw_name = &fields[0].0.to_owned();
    let Some(underscore_pos) = first_raw_name.find('_') else {
        return;
    };

    let candidate_prefix = &first_raw_name[..underscore_pos];

    if fields.iter().all(|(raw_name, _, _)| raw_name.starts_with(candidate_prefix)) {
        let prefix_upper = heck::AsUpperCamelCase(candidate_prefix).to_string();
        for (raw_name, rust_name, _) in fields.iter_mut() {
            if raw_name.starts_with(candidate_prefix) {
                if let Some(stripped) = rust_name.strip_prefix(&prefix_upper) {
                    // If we strip the prefix and the new name starts with a digit, revert.
                    if stripped.chars().next().map_or(false, |c| c.is_numeric()) {
                        continue;
                    }
                    *rust_name = stripped.to_owned();
                }
            }
        }
    }
}

fn replace_last(input: &str, pattern: &str, replacement: &str) -> String {
    if let Some(pos) = input.rfind(pattern) {
        let mut result = input.to_owned();
        result.replace_range(pos..pos+pattern.len(), replacement);
        result
    } else {
        input.to_owned()
    }
}