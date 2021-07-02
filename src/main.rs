#![windows_subsystem = "windows"]
mod mouse_state;
mod win;

use std::env;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::Result as IoResult;
use std::os::windows::ffi::OsStrExt;
use std::{mem, ptr};

use winapi::shared::minwindef::{TRUE, UINT};
use winapi::um::winuser;

use once_cell::sync::OnceCell;

use mouse_state::State;

const WINDOW_NAME: &str = "Mousey";

// There is no other reasonable way to pass additional data to SetWindowsHookExW
static WINDOW_HANDLE: OnceCell<usize> = OnceCell::new();
static OUTPUT_FILE: &str = "mouse_points.txt";

fn main() {
  unsafe {
    let window_name = OsStr::new(WINDOW_NAME)
      .encode_wide()
      .chain(Some(0).into_iter())
      .collect::<Vec<_>>();

    let file = get_file().unwrap();

    let mut mouse_state = State::new(file);
    let handle = win::create_window(&window_name, &mut mouse_state);

    winuser::RegisterHotKey(
      handle,
      win::TOGGLE_ID,
      win::TOGGLE_HOTKEY_MODIFIERS as UINT,
      win::TOGGLE_HOTKEY_VK as UINT,
    );

    WINDOW_HANDLE.set(handle as usize).unwrap();

    let hook = winuser::SetWindowsHookExW(
      winuser::WH_MOUSE_LL,
      Some(win::ll_mouse_hook),
      ptr::null_mut(),
      0,
    );

    let mut msg = mem::zeroed();

    loop {
      let recv = winuser::GetMessageW(&mut msg, ptr::null_mut(), 0, 0);

      if recv != TRUE || msg.message == winuser::WM_QUIT {
        break;
      }

      winuser::TranslateMessage(&msg);
      winuser::DispatchMessageW(&msg);
    }

    winuser::UnhookWindowsHookEx(hook);
  }
}

fn get_file() -> IoResult<File> {
  let mut cur_dir = env::current_dir()?;

  cur_dir.push(OUTPUT_FILE);

  if !cur_dir.exists() {
    File::create(&cur_dir)?;
  }

  OpenOptions::new()
    .read(true)
    .write(true)
    .append(true)
    .open(cur_dir)
}
