use winapi::shared::basetsd::LONG_PTR;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::shared::windowsx::{GET_X_LPARAM, GET_Y_LPARAM};
use winapi::um::winnt::{INT, LPCWSTR};
use winapi::um::winuser;

use std::time::Instant;
use std::{mem, ptr};

use crate::mouse_state::State as MouseState;
use crate::WINDOW_HANDLE;

pub const TOGGLE_ID: INT = 1 << 0;
pub const TOGGLE_HOTKEY_MODIFIERS: LONG_PTR = winuser::MOD_CONTROL | winuser::MOD_NOREPEAT;
pub const TOGGLE_HOTKEY_VK: INT = winuser::VK_F2;

pub unsafe extern "system" fn wnd_proc(
  hwnd: HWND,
  msg: UINT,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  // Restore pointer to state
  if msg == winuser::WM_NCCREATE {
    let create_struct = &*(lparam as *const winuser::CREATESTRUCTA);
    let user_data = create_struct.lpCreateParams as *const _ as LONG_PTR;

    winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, user_data);
  }

  let ms_ptr = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA) as *mut MouseState;

  if ms_ptr.is_null() {
    return winuser::DefWindowProcW(hwnd, msg, wparam, lparam);
  }

  let mouse_state = &mut *ms_ptr;

  match msg {
    winuser::WM_HOTKEY => {
      // Clear coordinates to prevent lapses in mouse points
      mouse_state.coordinates.clear();
      mouse_state.tracking = !mouse_state.tracking;
    }
    winuser::WM_MOUSEMOVE if mouse_state.tracking => {
      let now = Instant::now();
      // The points can be negative - https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-mousemove#remarks
      // but since the window is invisible, we only receive mouse moves from the low level hook
      // and those coordinates are retrieved from `GetCursorPos`, which seem to be correct
      let x = GET_X_LPARAM(lparam);
      let y = GET_Y_LPARAM(lparam);

      mouse_state.add_point((x, y), now);
    }
    _ => {}
  }

  winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
}

pub unsafe extern "system" fn ll_mouse_hook(code: INT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  // Per https://docs.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms644986(v=vs.85)?redirectedfrom=MSDN#parameters
  if code < 0 {
    return winuser::CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
  }

  if wparam as UINT == winuser::WM_MOUSEMOVE {
    let mut pt = mem::zeroed();

    winuser::GetCursorPos(&mut pt);

    // We need to convert the point to a LPARAM, but winapi-rs doesn't seem to have the MAKELPARAM macro
    let new_lparam = (pt.x & 0xFFFF) | (pt.y & 0xFFFF) << 16;

    winuser::PostMessageW(
      *WINDOW_HANDLE.get().unwrap() as *mut _,
      winuser::WM_MOUSEMOVE,
      wparam,
      new_lparam as LONG_PTR,
    );
  }

  winuser::CallNextHookEx(ptr::null_mut(), code, wparam, lparam)
}

pub unsafe fn create_window(class_name: &[u16], state: &mut MouseState) -> HWND {
  let class = winuser::WNDCLASSEXW {
    cbSize: mem::size_of::<winuser::WNDCLASSEXW>() as UINT,
    style: 0,
    lpfnWndProc: Some(wnd_proc),
    cbClsExtra: 0,
    cbWndExtra: 0,
    hInstance: ptr::null_mut(),
    hIcon: ptr::null_mut(),
    hCursor: ptr::null_mut(),
    hbrBackground: ptr::null_mut(),
    lpszMenuName: ptr::null(),
    lpszClassName: class_name.as_ptr(),
    hIconSm: ptr::null_mut(),
  };

  winuser::RegisterClassExW(&class);

  winuser::CreateWindowExW(
    0,
    class_name.as_ptr(),
    class_name.as_ptr() as LPCWSTR,
    0,
    winuser::CW_USEDEFAULT,
    winuser::CW_USEDEFAULT,
    winuser::CW_USEDEFAULT,
    winuser::CW_USEDEFAULT,
    ptr::null_mut(),
    ptr::null_mut(),
    ptr::null_mut(),
    state as *mut MouseState as *mut _,
  )
}
