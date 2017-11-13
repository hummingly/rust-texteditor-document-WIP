
mod lib;
use std::mem;
use std::ptr::{null, null_mut};
use document::lib::*;
use utils::{ToWide, error_msgbox, WinStruct};

use gdi32::GetDeviceCaps;
use kernel32::GetModuleHandleW;
use user32::*;
use winapi::*;

//STRUCTURES
pub struct Resources {
    pub render_target: *mut ID2D1HwndRenderTarget,
    pub brush: *mut ID2D1SolidColorBrush,
}

pub struct TextDocument {
    pub resources: Resources,
    pub hwnd: HWND,
    pub wtext: Vec<u16>,
    pub wtext_length: u32,
    pub d2d1_factory: *mut ID2D1Factory,
    pub dwrite_factory: *mut IDWriteFactory,
    pub text_format: *mut IDWriteTextFormat,
    pub text_layout: *mut IDWriteTextLayout,
    dpi_scale_x: i32,
    dpi_scale_y: i32,
}

impl TextDocument {
    pub fn initialized() -> Self {
        TextDocument {
            resources: Resources {
                render_target: null_mut(),
                brush: null_mut(),
            },
            hwnd: null_mut(),
            wtext: Vec::new(),
            wtext_length: 0,
            d2d1_factory: null_mut(),
            dwrite_factory: null_mut(),
            text_format: null_mut(),
            text_layout: null_mut(),
            dpi_scale_x: 0,
            dpi_scale_y: 0,
        }
    }
}

//D2D1 SETUP
fn create_factory_resources(doc: &mut TextDocument) {
    //D2D1 FACTORY
    let mut d2d1_factory: *mut c_void = null_mut();
    let factory_options = D2D1_FACTORY_OPTIONS { debugLevel: D2D1_DEBUG_LEVEL_NONE };

    if create_d2d1_factory(
        D2D1_FACTORY_TYPE_MULTI_THREADED,
        &UuidOfID2D1Factory,
        &factory_options as *const D2D1_FACTORY_OPTIONS,
        &mut d2d1_factory,
    ) != S_OK
    {
        error_msgbox("Could not create D2D1 factory.");
    } else {
        doc.d2d1_factory = d2d1_factory as *mut ID2D1Factory;
    }

    //DWRITE FACTORY
    let mut dwrite_factory: *mut IUnknown = null_mut();
    if create_dwrite_factory(
        DWRITE_FACTORY_TYPE_SHARED,
        &UuidOfIDWriteFactory,
        &mut dwrite_factory,
    ) != S_OK
    {
        error_msgbox("Could not create Dwrite factory.");
    } else {
        doc.dwrite_factory = dwrite_factory as *mut IDWriteFactory;
    }

    let text = "Hello World using DirectWrite!";
    doc.wtext_length = text.len() as u32;
    doc.wtext = text.to_wide();

    let dwrite_factory: &mut IDWriteFactory = unsafe { &mut *doc.dwrite_factory };

    //DWRITE TEXTFORMAT
    if dwrite_factory.create_text_format(
        "Palatino".to_wide().as_ptr(),
        null_mut(),
        DWRITE_FONT_WEIGHT_REGULAR,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        14.0,
        "en-us".to_wide().as_ptr(),
        &mut doc.text_format,
    ) != S_OK
    {
        error_msgbox("Could not create text format.");
    }

    let mut rect: RECT = WinStruct::default();
    unsafe { GetClientRect(doc.hwnd, &mut rect as *mut RECT) };

    let width = (rect.right / doc.dpi_scale_x) as f32;
    let height = (rect.bottom / doc.dpi_scale_y) as f32;

    //DWRITE TEXTLAYOUT
    if dwrite_factory.create_text_layout(
        doc.wtext.as_ptr(),
        doc.wtext_length,
        doc.text_format,
        width,
        height,
        &mut doc.text_layout,
    ) != S_OK
    {
        error_msgbox("Could not create text layout.");
    }
}

fn set_d2d_resources(doc: &mut TextDocument) {
    unsafe {
        if doc.d2d1_factory.is_null() {
            error_msgbox("There is no render target.");
        } else {
            let mut rect: RECT = WinStruct::default();

            GetClientRect(doc.hwnd, &mut rect as *mut RECT);

            let d2d_rect = D2D1_SIZE_U {
                width: (rect.right - rect.left) as u32,
                height: (rect.bottom - rect.top) as u32,
            };

            let render_properties: D2D1_RENDER_TARGET_PROPERTIES = WinStruct::default();

            let hwnd_render_properties = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd: doc.hwnd,
                pixelSize: d2d_rect,
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };

            let factory: &mut ID2D1Factory = &mut *doc.d2d1_factory;

            if factory.CreateHwndRenderTarget(
                &render_properties,
                &hwnd_render_properties,
                &mut doc.resources.render_target,
            ) != S_OK
            {
                error_msgbox("Could not create render target.");
            }

            let render_target: &mut ID2D1HwndRenderTarget = &mut *doc.resources.render_target;

            let black = Brush::black();

            if render_target.CreateSolidColorBrush(&black, null(), &mut doc.resources.brush) !=
                S_OK
            {
                error_msgbox("Could not create brush.");
            }
        }
    }
}

//RENDER METHOD
fn on_paint(doc: &mut TextDocument) -> HRESULT {
    unsafe {
        let d2d1_matrix: D2D1_MATRIX_3X2_F = WinStruct::default();
        let mut rect: RECT = WinStruct::default();
        GetClientRect(doc.hwnd, &mut rect as *mut RECT);

        let origin = D2D1_POINT_2F {
            x: (rect.left / doc.dpi_scale_x) as f32,
            y: (rect.top / doc.dpi_scale_y) as f32,
        };

        let white = Brush::white();

        let render = &mut *doc.resources.render_target;
        render.BeginDraw();

        render.SetTransform(&d2d1_matrix);

        render.Clear(&white);

        render.DrawTextLayout(
            origin,
            doc.text_layout,
            &mut **doc.resources.brush as *mut ID2D1Brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
        );

        render.EndDraw(null_mut(), null_mut())
    }
}

//RELEASE RESOURCES
fn safe_release(doc: &mut TextDocument) {
    unsafe {
        if !doc.resources.render_target.is_null() {
            (*doc.resources.brush).Release();
            (*doc.resources.render_target).Release();

            doc.resources.brush = null_mut();
            doc.resources.render_target = null_mut();
        }
    }
}

fn release_resources(doc: &mut TextDocument) {
    unsafe {
        safe_release(doc);

        if !doc.d2d1_factory.is_null() {
            (*doc.d2d1_factory).Release();
            doc.d2d1_factory = null_mut();
        }

        if !doc.dwrite_factory.is_null() {
            (*doc.dwrite_factory).Release();
            (*doc.text_format).Release();
            (*doc.text_layout).Release();

            doc.dwrite_factory = null_mut();
            doc.text_format = null_mut();
            doc.text_layout = null_mut();
        }
    }
}

//DPI SCALING
fn dpi_scaling(doc: &mut TextDocument) {
    unsafe {
        let screen = GetDC(null_mut());
        doc.dpi_scale_x = GetDeviceCaps(screen, LOGPIXELSX) / 96;
        doc.dpi_scale_y = GetDeviceCaps(screen, LOGPIXELSY) / 96;
    }
}

//WINDOW CREATION
pub fn init_text(mut doc: &mut TextDocument) {
    unsafe {
        let class = "TextArea".to_wide();
        let wndcl = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as UINT32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: mem::size_of::<TextDocument>() as INT32,
            hInstance: GetModuleHandleW(null_mut()),
            hIcon: 0 as HICON,
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: COLOR_WINDOWFRAME as HBRUSH,
            lpszMenuName: null(),
            lpszClassName: class.as_ptr() as *const u16,
            hIconSm: 0 as HICON,
        };

        match RegisterClassExW(&wndcl) {
            0 => {
                error_msgbox("Could not register class!");
                PostQuitMessage(0);
            }
            _atom => {
                RegisterClassExW(&wndcl);
                let hwnd = create_window();

                if hwnd.is_null() {
                    error_msgbox("Could not create window!");
                    PostQuitMessage(0);
                } else {
                    doc.hwnd = hwnd;
                }
                dpi_scaling(&mut doc);
                create_factory_resources(&mut doc);
                set_window_ptr(&mut doc);
            }
        }
    }
}

fn create_window() -> HWND {
    unsafe {
        let class = "TextArea".to_wide();
        let window = "Hello World!".to_wide();

        CreateWindowExW(
            WS_EX_COMPOSITED,
            class.as_ptr(),
            window.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            600,
            400,
            null_mut(),
            null_mut(),
            GetModuleHandleW(null_mut()),
            null_mut(),
        )
    }
}

//MESSAGE PROCESSING
unsafe extern "system" fn wndproc(
    hwnd: HWND,
    message: UINT32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let app_ptr = get_window_ptr(hwnd);
    let mut doc: &mut TextDocument = &mut *(app_ptr as *mut TextDocument);
    match message {
        WM_PAINT => {
            set_d2d_resources(doc);
            if on_paint(doc) == D2DERR_RECREATE_TARGET {
                safe_release(doc);
            }
            0
        }
        WM_SIZE => {
            let width = GET_X_LPARAM(lparam);
            let height = GET_Y_LPARAM(lparam);

            if !app_ptr.is_null() {
                let render_size = D2D_SIZE_U {
                    width: width as u32,
                    height: height as u32,
                };

                let render = &mut *doc.resources.render_target;
                render.Resize(&render_size);

                if !doc.text_layout.is_null() {
                    let text_layout = &mut *doc.text_layout;
                    text_layout.SetMaxWidth((width / doc.dpi_scale_x) as f32);
                    text_layout.SetMaxHeight((height / doc.dpi_scale_y) as f32);
                }
            }
            0
        }
        WM_DESTROY => {
            release_resources(&mut doc);
            PostQuitMessage(0);
            1
        }
        WM_NCDESTROY => {
            UnregisterClassW(
                "TextArea".to_wide().as_ptr() as *const u16,
                GetModuleHandleW(null_mut()),
            );
            0
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

pub fn message_loop() {
    unsafe {
        let mut msg: MSG = WinStruct::default();

        while GetMessageW(&mut msg as *mut MSG, 0 as HWND, 0, 0) != 0 {
            TranslateMessage(&msg as *const MSG);
            DispatchMessageW(&msg as *const MSG);
        }
    }
}

//ASSOCIATE STRUCTURES/DATA
fn get_window_ptr(hwnd: HWND) -> *mut TextDocument {
    unsafe { GetWindowLongPtrW(hwnd, 0) as *mut TextDocument }
}

fn set_window_ptr(doc: &mut TextDocument) {
    unsafe {
        SetWindowLongPtrW(doc.hwnd, 0, doc as *mut TextDocument as LONG_PTR);
    }
}
