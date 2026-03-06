use super::receiver::ReceiverKind;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{FnArg, Pat, Type};

/// Extracts a parameter name from a pattern.
pub fn extract_param_name(pat: &Pat) -> String {
    if let Pat::Ident(ident) = pat {
        ident.ident.to_string()
    } else {
        "param".to_string()
    }
}

/// Generates conversion code for a parameter from FFI type to Rust type.
pub fn generate_param_conversion(name: &str, ty: &Type, type_name: &str) -> Option<TokenStream> {
    let ident = format_ident!("{}", name);

    match type_name {
        "&str" => Some(quote! {
            // SAFETY: Caller guarantees the pointer is a valid null-terminated
            // C string. We checked for null above.
            let #ident = match std::ffi::CStr::from_ptr(#ident).to_str() {
                Ok(s) => s,
                Err(_) => return Default::default(),
            };
        }),
        "Entity" => Some(quote! {
            let #ident = crate::ecs::Entity::from_bits(#ident);
        }),
        "Key" => Some(quote! {
            let #ident: glfw::Key = match #ident {
                -1 => glfw::Key::Unknown,
                32 => glfw::Key::Space,
                39 => glfw::Key::Apostrophe,
                44 => glfw::Key::Comma,
                45 => glfw::Key::Minus,
                46 => glfw::Key::Period,
                47 => glfw::Key::Slash,
                48 => glfw::Key::Num0,
                49 => glfw::Key::Num1,
                50 => glfw::Key::Num2,
                51 => glfw::Key::Num3,
                52 => glfw::Key::Num4,
                53 => glfw::Key::Num5,
                54 => glfw::Key::Num6,
                55 => glfw::Key::Num7,
                56 => glfw::Key::Num8,
                57 => glfw::Key::Num9,
                59 => glfw::Key::Semicolon,
                61 => glfw::Key::Equal,
                65 => glfw::Key::A,
                66 => glfw::Key::B,
                67 => glfw::Key::C,
                68 => glfw::Key::D,
                69 => glfw::Key::E,
                70 => glfw::Key::F,
                71 => glfw::Key::G,
                72 => glfw::Key::H,
                73 => glfw::Key::I,
                74 => glfw::Key::J,
                75 => glfw::Key::K,
                76 => glfw::Key::L,
                77 => glfw::Key::M,
                78 => glfw::Key::N,
                79 => glfw::Key::O,
                80 => glfw::Key::P,
                81 => glfw::Key::Q,
                82 => glfw::Key::R,
                83 => glfw::Key::S,
                84 => glfw::Key::T,
                85 => glfw::Key::U,
                86 => glfw::Key::V,
                87 => glfw::Key::W,
                88 => glfw::Key::X,
                89 => glfw::Key::Y,
                90 => glfw::Key::Z,
                91 => glfw::Key::LeftBracket,
                92 => glfw::Key::Backslash,
                93 => glfw::Key::RightBracket,
                96 => glfw::Key::GraveAccent,
                161 => glfw::Key::World1,
                162 => glfw::Key::World2,
                256 => glfw::Key::Escape,
                257 => glfw::Key::Enter,
                258 => glfw::Key::Tab,
                259 => glfw::Key::Backspace,
                260 => glfw::Key::Insert,
                261 => glfw::Key::Delete,
                262 => glfw::Key::Right,
                263 => glfw::Key::Left,
                264 => glfw::Key::Down,
                265 => glfw::Key::Up,
                266 => glfw::Key::PageUp,
                267 => glfw::Key::PageDown,
                268 => glfw::Key::Home,
                269 => glfw::Key::End,
                280 => glfw::Key::CapsLock,
                281 => glfw::Key::ScrollLock,
                282 => glfw::Key::NumLock,
                283 => glfw::Key::PrintScreen,
                284 => glfw::Key::Pause,
                290 => glfw::Key::F1,
                291 => glfw::Key::F2,
                292 => glfw::Key::F3,
                293 => glfw::Key::F4,
                294 => glfw::Key::F5,
                295 => glfw::Key::F6,
                296 => glfw::Key::F7,
                297 => glfw::Key::F8,
                298 => glfw::Key::F9,
                299 => glfw::Key::F10,
                300 => glfw::Key::F11,
                301 => glfw::Key::F12,
                302 => glfw::Key::F13,
                303 => glfw::Key::F14,
                304 => glfw::Key::F15,
                305 => glfw::Key::F16,
                306 => glfw::Key::F17,
                307 => glfw::Key::F18,
                308 => glfw::Key::F19,
                309 => glfw::Key::F20,
                310 => glfw::Key::F21,
                311 => glfw::Key::F22,
                312 => glfw::Key::F23,
                313 => glfw::Key::F24,
                314 => glfw::Key::F25,
                320 => glfw::Key::Kp0,
                321 => glfw::Key::Kp1,
                322 => glfw::Key::Kp2,
                323 => glfw::Key::Kp3,
                324 => glfw::Key::Kp4,
                325 => glfw::Key::Kp5,
                326 => glfw::Key::Kp6,
                327 => glfw::Key::Kp7,
                328 => glfw::Key::Kp8,
                329 => glfw::Key::Kp9,
                330 => glfw::Key::KpDecimal,
                331 => glfw::Key::KpDivide,
                332 => glfw::Key::KpMultiply,
                333 => glfw::Key::KpSubtract,
                334 => glfw::Key::KpAdd,
                335 => glfw::Key::KpEnter,
                336 => glfw::Key::KpEqual,
                340 => glfw::Key::LeftShift,
                341 => glfw::Key::LeftControl,
                342 => glfw::Key::LeftAlt,
                343 => glfw::Key::LeftSuper,
                344 => glfw::Key::RightShift,
                345 => glfw::Key::RightControl,
                346 => glfw::Key::RightAlt,
                347 => glfw::Key::RightSuper,
                348 => glfw::Key::Menu,
                _ => glfw::Key::Unknown,
            };
        }),
        "MouseButton" => {
            let _ = ty;
            Some(quote! {
                let #ident = match #ident {
                    0 => glfw::MouseButton::Button1,
                    1 => glfw::MouseButton::Button2,
                    2 => glfw::MouseButton::Button3,
                    3 => glfw::MouseButton::Button4,
                    4 => glfw::MouseButton::Button5,
                    5 => glfw::MouseButton::Button6,
                    6 => glfw::MouseButton::Button7,
                    7 => glfw::MouseButton::Button8,
                    _ => glfw::MouseButton::Button1,
                };
            })
        }
        _ => None,
    }
}

/// Generates Vec2 reconstruction from flattened _x, _y params.
pub fn generate_vec2_reconstruction(name: &str) -> TokenStream {
    let (ident, x, y) = (
        format_ident!("{}", name),
        format_ident!("{}_x", name),
        format_ident!("{}_y", name),
    );
    quote! { let #ident = crate::core::math::Vec2::new(#x, #y); }
}

/// Builds the list of argument identifiers for calling the original method.
pub fn build_call_args(params: &[&FnArg]) -> Vec<Ident> {
    params
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pt) = arg {
                Some(format_ident!("{}", extract_param_name(&pt.pat)))
            } else {
                None
            }
        })
        .collect()
}

/// Builds a full method call expression with context lookup.
pub fn build_method_call(
    m: &Ident,
    recv: ReceiverKind,
    args: &[Ident],
    self_type: &str,
) -> TokenStream {
    match recv {
        ReceiverKind::None => {
            let ty = format_ident!("{}", self_type);
            quote! { #ty::#m(#(#args),*) }
        }
        _ => {
            let access = super::receiver::registry_access_for(recv);
            quote! {{
                if context_id.is_invalid() { return Default::default(); }
                let mut registry = match crate::core::context_registry::get_context_registry().lock() {
                    Ok(r) => r, Err(_) => return Default::default(),
                };
                #access
                context.#m(#(#args),*)
            }}
        }
    }
}

/// Builds a bare method call expression (no context lookup).
pub fn build_method_call_inner(m: &Ident, args: &[Ident]) -> TokenStream {
    quote! { #m(#(#args),*) }
}
