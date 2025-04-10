#[cfg(feature = "codegen")]
pub mod codegen;



extern crate resourcelib_sys;

// Optionally, bring the necessary items into scope
use resourcelib_sys::*;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::path::Path;
use thiserror::Error;
use crate::codegen::ResourceLibResource;

#[derive(Debug, Error)]
pub enum ResourceLibError {
    #[error("Failed to get supported resource types")]
    GetSupportedResourceTypes,

    #[error("Unable to use the resource type: {0}")]
    InvalidResourceType(String),

    #[error("Unable to path: {0} {1}")]
    InvalidPath(String, String),

    #[error("Null pointer encountered in function {0}")]
    NullPointer(&'static str),

    #[error("Conversion failed in function {0}")]
    ConversionFailed(&'static str),

    #[error("Resource converter function {0} returned an error")]
    ConverterFunctionError(&'static str),

    #[error("Resource generator function {0} returned an error")]
    GeneratorFunctionError(&'static str),

    #[error("Unknown WoaVersion variant")]
    UnknownWoaVersion,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[cfg(feature = "codegen")]
    #[error("serde json error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

#[derive(Debug, Copy, Clone)]
pub enum WoaVersion {
    HM2016,
    HM2,
    HM3,
}

fn prepare_resource_type(resource_type: &str) -> Result<CString, ResourceLibError> {
    if resource_type.len() != 4 {
        return Err(ResourceLibError::InvalidResourceType(
            resource_type.to_string(),
        ));
    }

    CString::new(resource_type)
        .map_err(|_| ResourceLibError::InvalidResourceType(resource_type.to_string()))
}

fn prepare_path_parameter<P: AsRef<Path>>(
    path: P,
    should_exit: bool,
) -> Result<CString, ResourceLibError> {
    let path_display = path.as_ref().to_string_lossy().into_owned();

    if should_exit && !path.as_ref().exists() {
        return Err(ResourceLibError::InvalidPath(
            path_display,
            "Path does not exist".to_string(),
        ));
    }
    CString::new(path_display.clone()).map_err(|_| {
        ResourceLibError::InvalidPath(path_display, "cannot convert to CString".to_string())
    })
}

pub struct ResourceLib;

impl ResourceLib {
    pub fn supported_resource_types(
        woa_version: WoaVersion,
    ) -> Result<Vec<String>, ResourceLibError> {
        let array_ptr = unsafe {
            match woa_version {
                WoaVersion::HM2016 => HM2016_GetSupportedResourceTypes(),
                WoaVersion::HM2 => HM2_GetSupportedResourceTypes(),
                WoaVersion::HM3 => HM3_GetSupportedResourceTypes(),
            }
        };

        if array_ptr.is_null() {
            return Err(ResourceLibError::GetSupportedResourceTypes);
        }

        let types =
            unsafe { std::slice::from_raw_parts((*array_ptr).Types, (*array_ptr).TypeCount) }
                .iter()
                .map(|&t| {
                    if t.is_null() {
                        Err(ResourceLibError::NullPointer("Types array element"))
                    } else {
                        unsafe {
                            CStr::from_ptr(t)
                                .to_str()
                                .map(|s| s.to_owned())
                                .map_err(ResourceLibError::Utf8Error)
                        }
                    }
                })
                .collect::<Result<Vec<String>, ResourceLibError>>()?;

        unsafe {
            match woa_version {
                WoaVersion::HM2016 => HM2016_FreeSupportedResourceTypes(array_ptr),
                WoaVersion::HM2 => HM2_FreeSupportedResourceTypes(array_ptr),
                WoaVersion::HM3 => HM3_FreeSupportedResourceTypes(array_ptr),
            };
        }

        Ok(types)
    }

    pub fn is_supported_resource_type(
        woa_version: WoaVersion,
        resource_type: &str,
    ) -> Result<bool, ResourceLibError> {
        let c_resource_type = prepare_resource_type(resource_type)?;
        let result = unsafe {
            match woa_version {
                WoaVersion::HM2016 => HM2016_IsResourceTypeSupported(c_resource_type.as_ptr()),
                WoaVersion::HM2 => HM2_IsResourceTypeSupported(c_resource_type.as_ptr()),
                WoaVersion::HM3 => HM3_IsResourceTypeSupported(c_resource_type.as_ptr()),
            }
        };

        Ok(result)
    }
}

/// Represents a resource converter.
#[derive(Debug, Clone)]
pub struct ResourceConverter {
    converter: *mut resourcelib_sys::ResourceConverter,
}

unsafe impl Send for ResourceConverter {}
unsafe impl Sync for ResourceConverter {}

impl ResourceConverter {
    /// Creates a new ResourceConverter for the specified resource type.
    pub fn new(version: WoaVersion, resource_type: &str) -> Result<Self, ResourceLibError> {
        let c_resource_type = prepare_resource_type(resource_type)?;

        let converter_ptr = unsafe {
            match version {
                WoaVersion::HM2016 => HM2016_GetConverterForResource(c_resource_type.as_ptr()),
                WoaVersion::HM2 => HM2_GetConverterForResource(c_resource_type.as_ptr()),
                WoaVersion::HM3 => HM3_GetConverterForResource(c_resource_type.as_ptr()),
            }
        };
        if converter_ptr.is_null() {
            Err(ResourceLibError::NullPointer("created converter"))
        } else {
            Ok(ResourceConverter {
                converter: converter_ptr,
            })
        }
    }

    /// Converts a resource file to a JSON file.
    pub fn resource_file_to_json_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        resource_file_path: P,
        output_file_path: Q,
    ) -> Result<bool, ResourceLibError> {
        let c_resource_file_path = prepare_path_parameter(resource_file_path, true)?;
        let c_output_file_path = prepare_path_parameter(output_file_path, false)?;

        unsafe {
            let func = (*self.converter).FromResourceFileToJsonFile.ok_or(
                ResourceLibError::ConverterFunctionError("FromResourceFileToJsonFile"),
            )?;
            Ok(func(
                c_resource_file_path.as_ptr(),
                c_output_file_path.as_ptr(),
            ))
        }
    }

    /// Converts a resource from memory to a JSON file.
    pub fn memory_to_json_file<P: AsRef<Path>>(
        &self,
        resource_data: &[u8],
        output_file_path: P,
    ) -> Result<bool, ResourceLibError> {
        let c_output_file_path = prepare_path_parameter(output_file_path, false)?;
        unsafe {
            let func = (*self.converter).FromMemoryToJsonFile.ok_or(
                ResourceLibError::ConverterFunctionError("FromMemoryToJsonFile"),
            )?;
            Ok(func(
                resource_data.as_ptr() as *const _,
                resource_data.len(),
                c_output_file_path.as_ptr(),
            ))
        }
    }

    /// Converts a resource from memory to a JSON string.
    pub fn memory_to_json_string(&self, resource_data: &[u8]) -> Result<String, ResourceLibError> {
        unsafe {
            let func = (*self.converter).FromMemoryToJsonString.ok_or(
                ResourceLibError::ConverterFunctionError("FromMemoryToJsonString"),
            )?;
            let json_string_ptr = func(resource_data.as_ptr() as *const _, resource_data.len());
            if json_string_ptr.is_null() {
                Err(ResourceLibError::NullPointer("json result string"))
            } else {
                let json_string = *json_string_ptr;
                let c_str = CStr::from_ptr(json_string.JsonData);
                let result = c_str.to_string_lossy().into_owned();
                let free_func = (*self.converter)
                    .FreeJsonString
                    .ok_or(ResourceLibError::ConverterFunctionError("FreeJsonString"))?;
                free_func(json_string_ptr);
                Ok(result)
            }
        }
    }

    /// Converts a resource file to a JSON string.
    pub fn resource_file_to_json_string<P: AsRef<Path>>(
        &self,
        resource_file_path: P,
    ) -> Result<String, ResourceLibError> {
        let c_resource_file_path = prepare_path_parameter(resource_file_path, true)?;
        unsafe {
            let func = (*self.converter).FromResourceFileToJsonString.ok_or(ResourceLibError::ConverterFunctionError("FromResourceFileToJsonString"))?;
            let json_string_ptr = func(c_resource_file_path.as_ptr());
            if json_string_ptr.is_null() {
                Err(ResourceLibError::NullPointer("json result string"))
            } else {
                let json_string = *json_string_ptr;
                let c_str = CStr::from_ptr(json_string.JsonData);
                let result = c_str.to_string_lossy().into_owned();
                let free_func = (*self.converter)
                    .FreeJsonString
                    .ok_or(ResourceLibError::ConverterFunctionError("FreeJsonString"))?;
                free_func(json_string_ptr);
                Ok(result)
            }
        }
    }
}

/// Represents a resource generator.
#[derive(Debug, Clone)]
pub struct ResourceGenerator {
    generator: *mut resourcelib_sys::ResourceGenerator,
}

unsafe impl Send for ResourceGenerator {}
unsafe impl Sync for ResourceGenerator {}

impl ResourceGenerator {
    /// Creates a new ResourceGenerator for the specified resource type.
    pub fn new(version: WoaVersion, resource_type: &str) -> Result<Self, ResourceLibError> {
        let c_resource_type = prepare_resource_type(resource_type)?;
        unsafe {
            let generator_ptr = match version {
                WoaVersion::HM2016 => HM2016_GetGeneratorForResource(c_resource_type.as_ptr()),
                WoaVersion::HM2 => HM2_GetGeneratorForResource(c_resource_type.as_ptr()),
                WoaVersion::HM3 => HM3_GetGeneratorForResource(c_resource_type.as_ptr()),
            };
            if generator_ptr.is_null() {
                Err(ResourceLibError::NullPointer("created generator"))
            } else {
                Ok(ResourceGenerator {
                    generator: generator_ptr,
                })
            }
        }
    }

    /// Generates a resource file from a JSON file.
    pub fn json_file_to_resource_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        json_file_path: P,
        resource_file_path: Q,
        generate_compatible: bool,
    ) -> Result<bool, ResourceLibError> {
        let c_json_file_path = prepare_path_parameter(json_file_path, true)?;
        let c_resource_file_path = prepare_path_parameter(resource_file_path, false)?;

        unsafe {
            let func = (*self.generator).FromJsonFileToResourceFile.ok_or(
                ResourceLibError::GeneratorFunctionError("FromJsonFileToResourceFile"),
            )?;
            Ok(func(
                c_json_file_path.as_ptr(),
                c_resource_file_path.as_ptr(),
                generate_compatible,
            ))
        }
    }

    /// Generates a resource file from a JSON string.
    pub fn json_string_to_resource_file<P: AsRef<Path>>(
        &self,
        json_str: &str,
        resource_file_path: P,
        generate_compatible: bool,
    ) -> Result<bool, ResourceLibError> {
        let c_resource_file_path = prepare_path_parameter(resource_file_path, true)?;
        unsafe {
            let func = (*self.generator).FromJsonStringToResourceFile.ok_or(
                ResourceLibError::GeneratorFunctionError("FromJsonStringToResourceFile"),
            )?;
            Ok(func(
                json_str.as_ptr() as *const _,
                json_str.len(),
                c_resource_file_path.as_ptr(),
                generate_compatible,
            ))
        }
    }

    /// Generates a resource in memory from a JSON file.
    pub fn json_file_to_resource_mem<P: AsRef<Path>>(
        &self,
        json_file_path: P,
        generate_compatible: bool,
    ) -> Result<Vec<u8>, ResourceLibError> {
        let c_json_file_path = prepare_path_parameter(json_file_path, true)?;
        unsafe {
            let func = (*self.generator).FromJsonFileToResourceMem.ok_or(ResourceLibError::GeneratorFunctionError("FromJsonFileToResourceMem"))?;
            let resource_mem_ptr = func(c_json_file_path.as_ptr(), generate_compatible);
            if resource_mem_ptr.is_null() {
                Err(ResourceLibError::NullPointer("created resource mem"))
            } else {
                let resource_mem = *resource_mem_ptr;
                let data_slice = std::slice::from_raw_parts(
                    resource_mem.ResourceData as *const u8,
                    resource_mem.DataSize,
                );
                let result = data_slice.to_vec();

                let free_func = (*self.generator).FreeResourceMem.ok_or(ResourceLibError::GeneratorFunctionError("FreeResourceMem"))?;
                free_func(resource_mem_ptr);
                Ok(result)
            }
        }
    }

    /// Generates a resource in memory from a JSON string.
    pub fn json_string_to_resource_mem(
        &self,
        json_str: &str,
        generate_compatible: bool,
    ) -> Result<Vec<u8>, ResourceLibError> {
        unsafe {
            let func = (*self.generator).FromJsonStringToResourceMem.ok_or(ResourceLibError::GeneratorFunctionError("FromJsonStringToResourceMem"))?;
            let resource_mem_ptr = func(
                json_str.as_ptr() as *const _,
                json_str.len(),
                generate_compatible,
            );
            if resource_mem_ptr.is_null() {
                Err(ResourceLibError::NullPointer("created resource mem"))
            } else {
                let resource_mem = *resource_mem_ptr;
                let data_slice = std::slice::from_raw_parts(
                    resource_mem.ResourceData as *const u8,
                    resource_mem.DataSize,
                );
                let result = data_slice.to_vec();

                let free_func = (*self.generator).FreeResourceMem.ok_or(ResourceLibError::GeneratorFunctionError("FreeResourceMem"))?;
                free_func(resource_mem_ptr);
                Ok(result)
            }
        }
    }
}
