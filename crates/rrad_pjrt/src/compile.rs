use std::marker::PhantomData;
use crate::ffi::pjrt_sys::*;
use crate::device::PJRTDevice;
use crate::error::PJRTError;
use crate::executable::PJRTLoadedExecutable;
use crate::loader::PjrtRuntime;
use std::ptr::null_mut;
use crate::client::PJRTClient;

pub struct PJRTCompiler<'rt, 'client> {
    pub rt: &'rt PjrtRuntime,
    pub raw: *mut PJRT_Client,
    _client: PhantomData<&'client PJRTClient<'rt>>,
}

impl<'rt, 'client> PJRTCompiler<'rt, 'client> {
    pub(crate) fn new(rt: &'rt PjrtRuntime, raw: *mut PJRT_Client) -> Self {
        Self { rt, raw, _client: PhantomData }
    }

    pub fn error(&self, msg: impl Into<String>) -> PJRTError<'rt> {
        PJRTError::invalid_arg(self.rt, msg)
    }

    fn raw_checked(&self) -> Result<*mut PJRT_Client, PJRTError<'rt>> {
        if self.raw.is_null() {
            Err(self.error("PJRT_Client for compiling is null"))
        } else {
            Ok(self.raw)
        }
    }

    pub fn compile_program(
        &self,
        program: &PJRT_Program,
        compile_options: &[u8],
    ) -> Result<PJRTLoadedExecutable<'rt, 'client>, PJRTError<'rt>> {
        let client = self.raw_checked()?;
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
            return Err(PJRTError::new(self.rt, err));
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
    ) -> Result<PJRTLoadedExecutable<'rt, 'client>, PJRTError<'rt>> {
        if program_code.is_empty() {
            return Err(self.error("program_code must not be empty"));
        }
        if format.is_empty() {
            return Err(self.error("format must not be empty"));
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
    ) -> Result<PJRTLoadedExecutable<'rt, 'client>, PJRTError<'rt>> {
        if format.is_empty() {
            return Err(self.error("format must not be empty"));
        }
        program.format = format.as_ptr() as *const libc::c_char;
        program.format_size = format.len();
        self.compile_program(program, compile_options)
    }

    pub fn addressable_devices(&self) -> Result<Vec<PJRTDevice<'rt, 'client>>, PJRTError<'rt>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_AddressableDevices
            .ok_or(self.error("PJRT_Client_AddressableDevices symbol not found"))?;

        let mut args = PJRT_Client_AddressableDevices_Args {
            struct_size: PJRT_Client_AddressableDevices_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client: raw,
            addressable_devices: std::ptr::null(),
            num_addressable_devices: 0,
        };

        let err = unsafe { f(&mut args) };

        if !err.is_null() {
            Err(PJRTError::new(self.rt, err))
        } else if args.num_addressable_devices == 0 {
            Ok(Vec::new())
        } else if args.addressable_devices.is_null() {
            Err(
                self.error(
                    "PJRT_Client_AddressableDevices returned null devices with nonzero count"))
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
