use std::ffi;
use std::mem;
use utils::{ToWide, error_msgbox};

use kernel32::*;
use winapi::*;

//DIRECT2D/DIRECTWRITE TYPES
#[allow(non_upper_case_globals)]
pub const UuidOfIDWriteFactory: IID = IID {
    Data1: 0xb859_ee5a,
    Data2: 0xd838,
    Data3: 0x4b5b,
    Data4: [0xa2, 0xe8, 0x1a, 0xdc, 0x7d, 0x93, 0xdb, 0x48],
};

type D2D1CreateFactoryFn = extern "system" fn(factoryType: D2D1_FACTORY_TYPE,
                                              riid: REFIID,
                                              pFactoryOptions: *const D2D1_FACTORY_OPTIONS,
                                              ppIFactory: *mut *mut c_void)
                                              -> HRESULT;

type DWriteCreateFactoryFn = extern "system" fn(factoryType: DWRITE_FACTORY_TYPE,
                                                iid: REFIID,
                                                factory: *mut *mut IUnknown)
                                                -> HRESULT;

//LIBRARY FUNCTIONS
pub fn load_library(lib: &str) -> HMODULE {
    unsafe {
        let lib_name = lib.to_wide();
        let mut library = GetModuleHandleW(lib_name.as_ptr());

        if !library.is_null() {
            return library;
        }
        library = LoadLibraryW(lib_name.as_ptr());
        library
    }
}

fn load_d2d1_create_factory(name: &str) -> D2D1CreateFactoryFn {
    unsafe {
        let lib = load_library("d2d1.dll");
        let procedure = ffi::CString::new(name).unwrap();
        let function_ptr = GetProcAddress(lib, procedure.as_ptr());

        if function_ptr.is_null() {
            error_msgbox("Could not load D2D1CreateFactory.");
        }
        mem::transmute::<_, D2D1CreateFactoryFn>(function_ptr)
    }
}

fn load_dwrite_create_factory(name: &str) -> DWriteCreateFactoryFn {
    unsafe {
        let lib = load_library("dwrite.dll");
        let procedure = ffi::CString::new(name).unwrap();
        let function_ptr = GetProcAddress(lib, procedure.as_ptr());

        if function_ptr.is_null() {
            error_msgbox("Could not load DWriteCreateFactory.");
        }
        mem::transmute::<_, DWriteCreateFactoryFn>(function_ptr)
    }
}

//SAFE WRAPPERS FOR CREATING FACTORIES
pub fn create_d2d1_factory(
    factory_type: D2D1_FACTORY_TYPE,
    riid: REFIID,
    p_factory_options: *const D2D1_FACTORY_OPTIONS,
    pp_factory: *mut *mut c_void,
) -> HRESULT {
    let function = load_d2d1_create_factory("D2D1CreateFactory");
    function(factory_type, riid, p_factory_options, pp_factory)
}

pub fn create_dwrite_factory(
    factory_type: DWRITE_FACTORY_TYPE,
    iid: REFIID,
    pp_factory: *mut *mut IUnknown,
) -> HRESULT {
    let function = load_dwrite_create_factory("DWriteCreateFactory");
    function(factory_type, iid, pp_factory)
}

pub trait DWriteFactory {
    #[allow(too_many_arguments)]
    fn create_text_format(
        &mut self,
        font_family_name: *const u16,
        font_collection: *mut IDWriteFontCollection,
        font_weight: DWRITE_FONT_WEIGHT,
        font_style: DWRITE_FONT_STYLE,
        font_stretch: DWRITE_FONT_STRETCH,
        font_size: f32,
        local_name: *const u16,
        text_format: *mut *mut IDWriteTextFormat,
    ) -> HRESULT;

    fn create_text_layout(
        &mut self,
        string: *const u16,
        string_length: u32,
        text_format: *mut IDWriteTextFormat,
        max_width: f32,
        max_height: f32,
        text_layout: *mut *mut IDWriteTextLayout,
    ) -> HRESULT;
}

impl DWriteFactory for IDWriteFactory {
    fn create_text_format(
        &mut self,
        font_family_name: *const u16,
        font_collection: *mut IDWriteFontCollection,
        font_weight: DWRITE_FONT_WEIGHT,
        font_style: DWRITE_FONT_STYLE,
        font_stretch: DWRITE_FONT_STRETCH,
        font_size: f32,
        local_name: *const u16,
        text_format: *mut *mut IDWriteTextFormat,
    ) -> HRESULT {
        unsafe {
            self.CreateTextFormat(
                font_family_name,
                font_collection,
                font_weight,
                font_style,
                font_stretch,
                font_size,
                local_name,
                text_format,
            )
        }
    }

    fn create_text_layout(
        &mut self,
        string: *const u16,
        string_length: u32,
        text_format: *mut IDWriteTextFormat,
        max_width: f32,
        max_height: f32,
        text_layout: *mut *mut IDWriteTextLayout,
    ) -> HRESULT {
        unsafe {
            self.CreateTextLayout(
                string,
                string_length,
                text_format,
                max_width,
                max_height,
                text_layout,
            )
        }
    }
}

pub trait Brush {
    fn solid_color(red: f32, green: f32, blue: f32) -> Self;
    fn black() -> Self;
    fn white() -> Self;
}

impl Brush for D2D1_COLOR_F {
    fn solid_color(red: f32, green: f32, blue: f32) -> Self {
        D2D1_COLOR_F {
            r: red,
            g: green,
            b: blue,
            a: 1.0,
        }
    }
    //PREDEFINED COLORS
    fn black() -> Self {
        D2D1_COLOR_F {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    fn white() -> Self {
        D2D1_COLOR_F {
            r: 255.0,
            g: 255.0,
            b: 255.0,
            a: 1.0,
        }
    }
}
