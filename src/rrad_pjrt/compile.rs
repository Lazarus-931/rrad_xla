use crate::pjrt_sys::*;
use crate::rrad_pjrt::device::PJRTDevice;
use crate::rrad_pjrt::error::PJRTError;
use crate::rrad_pjrt::executable::PJRTLoadedExecutable;
use crate::rrad_pjrt::loader::{error_to_string, PjrtRuntime};
use std::ptr::null_mut;

pub struct PJRTCompiler<'a> {
    rt: &'a PjrtRuntime,
    raw: *mut PJRT_Client,
}

impl<'a> PJRTCompiler<'a> {
    pub(crate) fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_Client) -> Self {
        Self { rt, raw }
    }

    pub fn error(&self, msg: impl Into<String>) -> PJRTError<'a> {
        PJRTError::invalid_arg(self.rt, msg)
    }

    fn raw_checked(&self) -> Result<*mut PJRT_Client, String> {
        if self.raw.is_null() {
            Err("PJRT_Client for compiling is null".to_string())
        } else {
            Ok(self.raw)
        }
    }

    pub fn compile_program(
        &self,
        program: &PJRT_Program,
        compile_options: &[u8],
    ) -> Result<PJRTLoadedExecutable<'a>, PJRTError<'a>> {
        let client = self
            .raw_checked()
            .or_else(|_| Err(self.error("PJRT_Client is null")))?;
        let mut program_local = *program;

        if program_local.struct_size == 0 {
            program_local.struct_size = std::mem::size_of::<PJRT_Program>();
        }
        if program_local.code_size > 0 && program_local.code.is_null() {
            return Err(self.error("PJRT_Program.code is null but code_size is nonzero"));
        }
        if program_local.format_size > 0 && program_local.format.is_null() {
            return Err(self.error("PJRT_Program.format is null but format_size is nonzero"));
        }

        let client_compile = self
            .rt
            .api()
            .PJRT_Client_Compile
            .ok_or(self.error("PJRT_Client_Compile symbol not found"))?;

        let (compile_options_ptr, compile_options_size) = if compile_options.is_empty() {
            (std::ptr::null(), 0usize)
        } else {
            (
                compile_options.as_ptr() as *const libc::c_char,
                compile_options.len(),
            )
        };

        let mut args = PJRT_Client_Compile_Args {
            struct_size: PJRT_Client_Compile_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            program: &program_local,
            compile_options: compile_options_ptr,
            compile_options_size,
            executable: null_mut(),
        };

        let err = unsafe { client_compile(&mut args) };

        if !err.is_null() {
            return Err(self.error("PJRT_Client_Compile is null"));
        }
        if args.executable.is_null() {
            return Err(self.error("PJRT_Client_Compile returned null executable"));
        }

        Ok(PJRTLoadedExecutable::new(self.rt, args.executable))
    }

    pub fn compile(
        &self,
        program_code: &str,
        format: &str,
        compile_options: &[u8],
    ) -> Result<PJRTLoadedExecutable<'a>, PJRTError<'a>> {
        if program_code.is_empty() {
            return Err(self.error("program_code must not be empty")).into();
        }
        if format.is_empty() {
            return Err(self.error("format must not be empty")).into();
        }

        let program = PJRT_Program {
            // Bindings constant name is corrupted; use ABI size directly.
            struct_size: std::mem::size_of::<PJRT_Program>(),
            extension_start: std::ptr::null_mut(),
            code: program_code.as_ptr() as *mut libc::c_char,
            code_size: program_code.len(),
            format: format.as_ptr() as *const libc::c_char,
            format_size: format.len(),
        };

        self.compile_program(&program, compile_options)
    }

    pub fn compile_program_with_format(
        &self,
        program: &mut PJRT_Program,
        format: &str,
        compile_options: &[u8],
    ) -> Result<PJRTLoadedExecutable<'a>, PJRTError<'a>> {
        if format.is_empty() {
            return Err(self.error("format must not be empty"));
        }
        program.format = format.as_ptr() as *const libc::c_char;
        program.format_size = format.len();
        self.compile_program(program, compile_options)
    }

    pub fn addressable_devices(&self) -> Result<Vec<PJRTDevice<'a>>, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_AddressableDevices
            .ok_or("PJRT_Client_AddressableDevices symbol not found")?;

        let mut args = PJRT_Client_AddressableDevices_Args {
            struct_size: PJRT_Client_AddressableDevices_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client: raw,
            addressable_devices: std::ptr::null(),
            num_addressable_devices: 0,
        };

        let err = unsafe { f(&mut args) };

        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err))
        } else if args.num_addressable_devices == 0 {
            Ok(Vec::new())
        } else if args.addressable_devices.is_null() {
            Err(
                "PJRT_Client_AddressableDevices returned null devices with nonzero count"
                    .to_string(),
            )
        } else {
            let bytes = unsafe {
                std::slice::from_raw_parts(args.addressable_devices, args.num_addressable_devices)
            };
            let devices = bytes
                .iter()
                .map(|raw_device| PJRTDevice::new(self.rt, *raw_device))
                .collect();
            Ok(devices)
        }
    }
}
